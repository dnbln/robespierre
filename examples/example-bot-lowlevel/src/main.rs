use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use robespierre_cache::{Cache, CacheConfig, CommitToCache, HasCache};
use robespierre_events::{Authentication, Connection, RawEventHandler};
use robespierre_http::{Http, HttpAuthentication};
use robespierre_models::{autumn::AutumnTag, channel::{MessageContent, ReplyData}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let token = std::env::var("TOKEN")
        .expect("Cannot get token; set environment variable TOKEN=... and run again");

    let http = Http::new(HttpAuthentication::BotToken { token: &token }).await?;
    let connection = Connection::connect(Authentication::Bot { token: &token }).await?;

    let cache = Cache::new(CacheConfig::default());

    let context = Context(Arc::new(http), cache, Arc::new(AtomicUsize::new(0)));

    let handler = Handler;

    connection.run(context, handler).await?;

    Ok(())
}

#[derive(Clone)]
struct Context(Arc<Http>, Arc<Cache>, Arc<AtomicUsize>);

impl HasCache for Context {
    fn get_cache(&self) -> Option<&Cache> {
        Some(&self.1)
    }
}

impl robespierre_events::Context for Context {
    fn set_messanger(self, _messanger: robespierre_events::ConnectionMessanger) -> Self {
        // ignore for now
        self
    }
}

#[derive(Copy, Clone)]
struct Handler;

#[async_trait::async_trait]
impl RawEventHandler for Handler {
    type Context = Context;

    async fn handle(
        self,
        ctx: Self::Context,
        event: robespierre_models::events::ServerToClientEvent,
    ) {
        event.commit_to_cache_ref(&ctx).await;

        if let robespierre_models::events::ServerToClientEvent::Message { message } = event {
            if matches!(&message.content, MessageContent::Content(s) if s == "Hello") {
                let author = ctx.0.fetch_user(message.author).await.unwrap();
                let channel = ctx.0.fetch_channel(message.channel).await.unwrap();
                let server = if let Some(server) = channel.server_id() {
                    Some(ctx.0.fetch_server(server).await.unwrap())
                } else {
                    None
                };

                // let _session = message.channel.start_typing(ctx);
                // tokio::time::sleep(std::time::Duration::from_secs(10)).await;

                let att_id = ctx
                    .0
                    .upload_autumn(
                        AutumnTag::Attachments,
                        "help".to_string(),
                        "help me".to_string().into_bytes(),
                    )
                    .await
                    .unwrap();

                let _ = ctx
                    .0
                    .send_message(
                        message.channel,
                        format!(
                            "Hello <@{}> from <#{}>{}",
                            author.id,
                            channel.id(),
                            server
                                .map_or_else(Default::default, |it| format!(" in {}", it.name))
                        ),
                        rusty_ulid::generate_ulid_string(),
                        vec![att_id],
                        vec![ReplyData {
                            id: message.id,
                            mention: true,
                        }],
                    )
                    .await;
            }

            // framework commands
            if matches!(&message.content, MessageContent::Content(s) if s == "!ping" || s == "!pong") {
                let num = ctx.2.fetch_add(1, Ordering::Relaxed);

                let _ = ctx
                    .0
                    .send_message(
                        message.channel,
                        "Who pinged me?!",
                        rusty_ulid::generate_ulid_string(),
                        vec![],
                        vec![ReplyData {
                            id: message.id,
                            mention: false,
                        }],
                    )
                    .await;
                let _ = ctx
                    .0
                    .send_message(
                        message.channel,
                        format!("I got {} pings since I came online", num),
                        rusty_ulid::generate_ulid_string(),
                        vec![],
                        vec![ReplyData {
                            id: message.id,
                            mention: false,
                        }],
                    )
                    .await;
            }
        }
    }
}
