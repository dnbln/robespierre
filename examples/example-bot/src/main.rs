use robespierre::model::mention::Mentionable;
use robespierre::model::ChannelIdExt;
use robespierre::{async_trait, model::MessageExt, Context, EventHandler, EventHandlerWrap};
use robespierre::{Authentication, CacheWrap};
use robespierre_cache::CacheConfig;
use robespierre_events::Connection;
use robespierre_http::Http;
use robespierre_models::autumn::AutumnTag;
use robespierre_models::channel::{Message, ReplyData};
use robespierre_models::id::{ChannelId, ServerId, UserId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let token = std::env::var("TOKEN")
        .expect("Cannot get token; set environment variable TOKEN=... and run again");
    let auth = Authentication::bot(token);

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

        let _session = message.channel.start_typing(&ctx);
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        let att_id = ctx
            .http
            .upload_autumn(
                AutumnTag::Attachments,
                "help".to_string(),
                "help me".to_string().into_bytes(),
            )
            .await
            .unwrap();

        let _ = message
            .channel
            .send_message(&ctx, |msg| {
                msg.content(format!(
                    "Hello {} from {}{}",
                    author.mention(),
                    channel.mention(),
                    server.map_or_else(Default::default, |it| format!(" in {}", it.name))
                ))
                .reply(ReplyData {
                    id: message.id,
                    mention: true,
                })
                .attachment(att_id)
            })
            .await;
    }

    async fn on_server_member_join(&self, ctx: Context, server: ServerId, user: UserId) {
        if server != "01FEFZGF62HTX6MVYBTZ9F1K1S" {
            return;
        }

        let channel = "01FEFZXHDQMD5ESK0XXW93JM5R".parse::<ChannelId>().unwrap();

        channel
            .send_message(&ctx, |msg| {
                msg.content(format!("Welcome {}!", user.mention()))
            })
            .await
            .unwrap();
    }
}
