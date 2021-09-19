//! Self bot that sends "Welcome" whenever someone joins a server,
//! in the channel where the "User joined" message is sent.
//!
//! Base: example-bot-lowlevel
//! Book chapter: None

use std::sync::Arc;

use robespierre_cache::{Cache, CacheConfig, CommitToCache, HasCache};
use robespierre_client_core::{Authentication, model::ChannelIdExt};
use robespierre_events::{Connection, ConnectionMessage, ConnectionMessanger, RawEventHandler};
use robespierre_http::{HasHttp, Http};
use robespierre_models::{
    channels::{MessageContent, SystemMessage},
    events::ServerToClientEvent,
    id::UserId,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let token = std::env::var("TOKEN")
        .expect("Cannot get token; set environment variable TOKEN=... and run again");

    let auth = Authentication::user(token);

    let http = Http::new(&auth).await?;
    let connection = Connection::connect(&auth).await?;

    let acc = http.fetch_account().await?;

    let cache = Cache::new(CacheConfig::default());

    let context = Context(Arc::new(http), cache, None, acc.id);

    let handler = Handler;

    connection
        .run(context, handler)
        .await
        .expect("error while running the connection");

    Ok(())
}

#[derive(Clone)]
struct Context(Arc<Http>, Arc<Cache>, Option<ConnectionMessanger>, UserId);

impl HasCache for Context {
    fn get_cache(&self) -> Option<&Cache> {
        Some(&self.1)
    }
}

impl HasHttp for Context {
    fn get_http(&self) -> &Http {
        &self.0
    }
}

impl robespierre_events::Context for Context {
    fn set_messanger(mut self, messanger: robespierre_events::ConnectionMessanger) -> Self {
        self.2 = Some(messanger);
        self
    }
}

#[derive(Copy, Clone)]
struct Handler;

#[async_trait::async_trait]
impl RawEventHandler for Handler {
    type Context = Context;

    async fn handle(self, ctx: Self::Context, event: ServerToClientEvent) {
        event.commit_to_cache_ref(&ctx).await;

        if let ServerToClientEvent::Message { message } = event {
            match &message.content {
                MessageContent::SystemMessage(SystemMessage::UserJoined { id }) => {
                    if *id == ctx.3 {
                        // don't welcome yourself
                        return;
                    }

                    if message.channel != "01FDFCXPJ92A3MJ01DDVFYWR8M" {
                        return;
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

                    ctx.2
                        .as_ref()
                        .unwrap()
                        .send(ConnectionMessage::StartTyping {
                            channel: message.channel,
                        });

                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

                    let _ = message
                        .channel
                        .send_message(&ctx, |m| m.content("Welcome"))
                        .await;

                    ctx.2.as_ref().unwrap().send(ConnectionMessage::StopTyping {
                        channel: message.channel,
                    });
                }
                _ => {}
            }
        }
    }
}
