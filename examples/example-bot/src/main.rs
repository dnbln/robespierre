use robespierre::model::mention::Mentionable;
use robespierre::model::ChannelIdExt;
use robespierre::{async_trait, model::MessageExt, Context, EventHandler, EventHandlerWrap};
use robespierre::{Authentication, CacheWrap};
use robespierre_cache::CacheConfig;
use robespierre_events::Connection;
use robespierre_http::Http;
use robespierre_models::channel::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let token = std::env::var("TOKEN")
        .expect("Cannot get token; set environment variable TOKEN=... and run again");
    let auth = Authentication::Bot { token };

    let http = Http::new(&auth).await?;
    let connection = Connection::connect(&auth).await?;

    let ctx = Context::new(http).with_cache(CacheConfig::default());

    connection
        .run(ctx, EventHandlerWrap::new(CacheWrap::new(Handler)))
        .await?;

    Ok(())
}

#[derive(Copy, Clone)]
struct Handler;

#[async_trait]

impl EventHandler for Handler {
    async fn on_message(&self, ctx: Context, message: Message) {
        if message.content != "Hello" {
            return;
        }

        let author = message.author(&ctx).await.unwrap();
        let channel = message.channel(&ctx).await.unwrap();
        let server = message.server(&ctx).await.unwrap();

        message.channel.start_typing(&ctx);
        tokio::time::sleep(std::time::Duration::new(2, 500_000_000)).await;

        message.channel.start_typing(&ctx);
        tokio::time::sleep(std::time::Duration::new(2, 500_000_000)).await;

        message.channel.start_typing(&ctx);
        tokio::time::sleep(std::time::Duration::new(2, 500_000_000)).await;

        let _ = message
            .reply(
                &ctx,
                format!(
                    "Hello {} from {}{}",
                    author.mention(),
                    channel.mention(),
                    server.map_or_else(Default::default, |it| format!(" in {}", it.name))
                ),
            )
            .await;

        message.channel.stop_typing(&ctx);
    }
}
