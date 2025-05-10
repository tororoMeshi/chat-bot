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

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        gemini_model, gemini_api_key
    );

    let req_body = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart {
                text: input.to_string(),
            }],
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
