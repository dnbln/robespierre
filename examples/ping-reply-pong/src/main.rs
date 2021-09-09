// src/main.rs
use robespierre::model::MessageExt;
use robespierre::Authentication;
use robespierre::CacheWrap;
use robespierre::Context;
use robespierre::EventHandlerWrap;
use robespierre_cache::CacheConfig;
use robespierre_events::Connection;
use robespierre_http::Http;
use robespierre_models::channel::Message;
use robespierre_models::channel::MessageContent;
use robespierre_models::events::ReadyEvent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let token = std::env::var("TOKEN")
        .expect("Cannot get token; set environment variable TOKEN=... and run again");
    let auth = Authentication::bot(token);
    let http = Http::new(&auth).await?;
    let connection = Connection::connect(&auth).await?;
    let context = Context::new(http, robespierre::typemap::ShareMap::custom())
        .with_cache(CacheConfig::default());
    let handler = Handler;
    let handler = CacheWrap::new(EventHandlerWrap::new(handler));
    connection.run(context, handler).await?;

    Ok(())
}

#[derive(Clone)]
struct Handler;

#[robespierre::async_trait]
impl robespierre::EventHandler for Handler {
    async fn on_ready(&self, _ctx: Context, _ready: ReadyEvent) {
        tracing::info!("We're ready!");
    }

    async fn on_message(&self, ctx: Context, message: Message) {
        if matches!(&message.content, MessageContent::Content(s) if s == "ping") {
            let _ = message.reply(&ctx, "pong").await;
        }
    }
}
