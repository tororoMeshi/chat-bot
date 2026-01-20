use anyhow::{anyhow, Context as AnyhowContext, Result};
use once_cell::sync::Lazy;
use poise::serenity_prelude as serenity;
use reqwest as external_reqwest; // ← 明示的に名前変更
use serde::{Deserialize, Serialize};
use serenity::{builder::GetMessages, Client, GatewayIntents};
use std::env;
use std::fs;
use tracing::{error, info, warn};

static PROMPT_TEMPLATE: Lazy<String> =
    Lazy::new(|| match fs::read_to_string("/config/prompt_q.md") {
        Ok(content) => content,
        Err(err) => {
            warn!(
                error = %err,
                "/config/prompt_q.md が読み込めませんでした。空文字を使用します。"
            );
            String::new()
        }
    });

struct Data;
type Error = anyhow::Error;

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContentReply,
}

#[derive(Deserialize)]
struct GeminiContentReply {
    parts: Vec<GeminiPartReply>,
}

#[derive(Deserialize)]
struct GeminiPartReply {
    text: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    if let Err(err) = dotenv::dotenv() {
        warn!(error = %err, "dotenv の読み込みに失敗しました");
    }

    let token = env::var("DISCORD_TOKEN").context("DISCORD_TOKEN not set")?;
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions::<Data, Error> {
            commands: vec![q()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("/".to_string()),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|_, ready, _| {
            Box::pin(async move {
                info!(user = %ready.user.name, "discord connected");
                Ok(Data)
            })
        })
        .build();

    let mut client = Client::builder(token, intents)
        .framework(framework)
        .await
        .context("Error creating client")?;

    if let Err(err) = client.start().await {
        error!(error = %err, "Client error");
        return Err(err.into());
    }

    Ok(())
}

#[poise::command(prefix_command)]
async fn q(ctx: poise::Context<'_, Data, Error>, #[rest] input: Option<String>) -> Result<()> {
    let input_text = input.unwrap_or_default();
    if let Err(err) = q_impl(ctx, input_text).await {
        error!(error = %err, "q command failed");
        return Err(err);
    }
    Ok(())
}

async fn q_impl(ctx: poise::Context<'_, Data, Error>, input: String) -> Result<()> {
    let msg = match ctx {
        poise::Context::Prefix(prefix_ctx) => prefix_ctx.msg,
        poise::Context::Application(_) => {
            return Err(anyhow!("スラッシュコマンドは未対応です"));
        }
    };
    let serenity_ctx = ctx.serenity_context();
    let input = input.trim();
    info!(
        user = %msg.author.name,
        channel = %msg.channel_id,
        input_len = input.len(),
        "q command received"
    );

    let gemini_api_key = env::var("GEMINI_API_KEY").context("GEMINI_API_KEY not set")?;
    let gemini_model = env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-pro".to_string());
    let history_limit: usize = match env::var("CHAT_HISTORY_LIMIT") {
        Ok(value) => value
            .parse()
            .context("CHAT_HISTORY_LIMIT must be a valid usize")?,
        Err(_) => 3,
    };

    // Botの現在の名前（Discordから取得）
    let bot_name = serenity_ctx
        .http
        .get_current_user()
        .await
        .context("Failed to fetch current bot user")?
        .name
        .clone();

    // チャンネル履歴からメッセージ取得
    let requested_limit = history_limit.saturating_add(1);
    let limit = match u8::try_from(requested_limit) {
        Ok(value) => value,
        Err(_) => {
            warn!(
                requested_limit,
                "CHAT_HISTORY_LIMIT が大きすぎるため上限に丸めます"
            );
            u8::MAX
        }
    };
    let messages = msg
        .channel_id
        .messages(serenity_ctx, GetMessages::new().limit(limit))
        .await
        .context("Failed to retrieve channel messages")?
        .into_iter()
        .filter(|m| !m.content.starts_with("/q"))
        .filter(|m| m.id != msg.id)
        .collect::<Vec<_>>();

    // 履歴を古い順に整列 & Bot の発言には "Bot:" を強制付与
    let mut history_lines = messages
        .into_iter()
        .rev()
        .map(|m| {
            let speaker = if m.author.name == bot_name {
                "Bot"
            } else {
                &m.author.name
            };
            format!("{}: {}", speaker, m.content.trim())
        })
        .collect::<Vec<_>>();

    // 現在の入力も履歴に追加
    history_lines.push(format!("{}: {}", msg.author.name, input));

    // プロンプトの構成：人格指針 + 会話履歴 + 指示
    let full_prompt = format!(
        "{}\n\n以下は直近の会話です：\n{}\n\nBot: ",
        *PROMPT_TEMPLATE,
        history_lines.join("\n")
    );

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        gemini_model, gemini_api_key
    );

    let req_body = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart { text: full_prompt }],
        }],
    };

    let client = external_reqwest::Client::new();
    let res = client
        .post(&url)
        .json(&req_body)
        .send()
        .await
        .context("Failed to send request to Gemini")?;

    let status = res.status();
    if !status.is_success() {
        let body = res
            .text()
            .await
            .unwrap_or_else(|_| "レスポンス本文の取得に失敗しました".to_string());
        error!(
            status = %status,
            body = %body,
            "Gemini returned an error status"
        );
        if status == external_reqwest::StatusCode::TOO_MANY_REQUESTS {
            let user_message =
                "Gemini APIの利用上限に達しています。時間をおいて再実行してください。";
            msg.channel_id
                .say(serenity_ctx, user_message)
                .await
                .context("Failed to send 429 message to channel")?;
        }
        return Err(anyhow!("Gemini returned an error status: {}", status));
    }

    let json: GeminiResponse = res
        .json()
        .await
        .context("Failed to parse Gemini response")?;
    let reply = json
        .candidates
        .first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.clone())
        .unwrap_or("回答が取得できませんでした。".to_string());

    msg.channel_id
        .say(serenity_ctx, reply)
        .await
        .context("Failed to send message to channel")?;
    Ok(())
}
