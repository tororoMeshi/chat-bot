use once_cell::sync::Lazy;
use reqwest as external_reqwest; // ← 明示的に名前変更
use serde::{Deserialize, Serialize};
use serenity::{
    async_trait,
    framework::standard::{
        macros::{command, group},
        CommandResult, StandardFramework,
    },
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::env;
use std::fs;

static PROMPT_TEMPLATE: Lazy<String> = Lazy::new(|| {
    fs::read_to_string("/config/prompt_q.md").unwrap_or_else(|_| {
        eprintln!("⚠️ /config/prompt_q.md が読み込めませんでした。空文字を使用します。");
        "".to_string()
    })
});

#[group]
#[commands(q)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

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
async fn main() {
    dotenv::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("/"))
        .group(&GENERAL_GROUP); // ← ここも必須

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = serenity::Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[command]
async fn q(ctx: &Context, msg: &Message) -> CommandResult {
    let input = msg.content.trim_start_matches("/q").trim();
    let gemini_api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");
    let gemini_model = env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-pro".to_string());
    let history_limit: usize = env::var("CHAT_HISTORY_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);

    // Botの現在の名前（Discordから取得）
    let bot_name = ctx.http.get_current_user().await?.name;

    // チャンネル履歴からメッセージ取得
    let messages = msg
        .channel_id
        .messages(&ctx.http, |retriever| {
            retriever.limit((history_limit + 1) as u64)
        })
        .await?
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
    let res = client.post(&url).json(&req_body).send().await?;

    let json: GeminiResponse = res.json().await?;
    let reply = json
        .candidates
        .first()
        .and_then(|c| c.content.parts.first())
        .map(|p| p.text.clone())
        .unwrap_or("回答が取得できませんでした。".to_string());

    msg.channel_id.say(&ctx.http, reply).await?;
    Ok(())
}
