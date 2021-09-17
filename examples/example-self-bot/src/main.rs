//! Self bot that sends "Welcome" whenever someone joins a server,
//! in the channel where the "User joined" message is sent.
//!
//! Base: example-bot-lowlevel
//! Book chapter: None

use std::sync::Arc;

use robespierre_cache::{Cache, CacheConfig, CommitToCache, HasCache};
use robespierre_events::{
    Authentication, Connection, ConnectionMessage, ConnectionMessanger, RawEventHandler,
};
use robespierre_http::{Http, HttpAuthentication};
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

    let http = Http::new(HttpAuthentication::UserSession {
        session_token: &token,
    })
    .await?;
    let connection = Connection::connect(Authentication::User {
        session_token: &token,
    })
    .await?;

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

                    if let Err(err) = ctx
                        .0
                        .send_message(
                            message.channel,
                            "Welcome",
                            rusty_ulid::generate_ulid_string(),
                            vec![],
                            vec![],
                        )
                        .await
                    {
                        tracing::error!("Error {}; full: {:?}", err, err);
                    }

                    ctx.2.as_ref().unwrap().send(ConnectionMessage::StopTyping {
                        channel: message.channel,
                    });
                }
                _ => {}
            }
        }
    }
}
