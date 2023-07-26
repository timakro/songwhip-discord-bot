use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serenity::client::{ClientBuilder, Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::gateway::{GatewayIntents, Ready};
use std::env;

#[derive(Serialize, Deserialize)]
struct URL {
    url: String,
}

struct Handler {
    re: Regex,
    client: Client,
}

impl Handler {
    fn new() -> Self {
        Self {
            re: Regex::new(r"https://([^\s/.]+\.)*(spotify\.com|music\.apple\.com|youtube\.com|youtu\.be|tidal\.com|music\.amazon\.[^\s/.]+|pandora\.com|soundcloud\.com|deezer\.com|qobuz\.com|napster\.com)/\S+").unwrap(),
            client: Client::new(),
        }
    }
}

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.is_own(&ctx.cache) {
            if let Some(m) = self.re.find(&msg.content) {
                // https://songwhip.com/faq#does-songwhip-have-an-api
                let res = self
                    .client
                    .post("https://songwhip.com/")
                    .json(&URL {
                        url: m.as_str().to_string(),
                    })
                    .send()
                    .await
                    .unwrap();

                if res.status().is_success() {
                    let url = res.json::<URL>().await.unwrap().url;

                    // Wrap in <> to disable auto-embed
                    msg.reply(&ctx.http, format!("<{url}>")).await.unwrap();
                }
            }
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env if it exists
    dotenvy::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").unwrap();
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = ClientBuilder::new(&token, intents)
        .event_handler(Handler::new())
        .await
        .unwrap();

    client.start().await.unwrap();
}
