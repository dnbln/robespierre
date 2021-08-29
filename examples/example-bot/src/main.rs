use robespierre::{Context, EventHandler, EventHandlerWrap};
use robespierre_events::{Authentication, Connection};
use robespierre_http::{Http, HttpAuthentication};
use robespierre_models::channel::Message;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let token = std::env::var("TOKEN")
        .expect("Cannot get token; set environment variable TOKEN=... and run again");

    let http = Http::new(HttpAuthentication::BotToken {
        token: token.clone(),
    });

    let connection = Connection::connect(Authentication::Bot { token }).await?;

    let ctx = Context::new(http);

    connection.run(ctx, EventHandlerWrap::new(Handler)).await?;

    Ok(())
}

#[derive(Copy, Clone)]
struct Handler;

#[robespierre::async_trait]

impl EventHandler for Handler {
    async fn on_ready(&self, _ctx: Context, _ready: robespierre_events::ReadyEvent) {
        tracing::info!("I am ready");
    }

    async fn on_message(&self, ctx: Context, message: Message) {
        if message.content != "Hello" {
            return;
        }

        let _ = ctx
            .http
            .send_message(
                &message.channel,
                format!("Hello <@{}>", &message.author),
                rusty_ulid::generate_ulid_string(),
                vec![],
                vec![],
            )
            .await;
    }
}
