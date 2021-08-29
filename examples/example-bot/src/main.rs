use std::{future::Future, pin::Pin, sync::Arc};

use robespierre_events::{Authentication, Connection, EventHandler, ServerToClientEvent};
use robespierre_http::{Http, HttpAuthentication};
use robespierre_models::{
    channel::Message,
    id::{IdString, UserId},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let token = std::env::var("TOKEN")
        .expect("Cannot get token; set environment variable TOKEN=... and run again");

    let http = Http::new(HttpAuthentication::BotToken {
        token: token.clone(),
    });

    let http = Arc::new(http);

    let connection = Connection::connect(Authentication::Bot { token }).await?;

    let handler = Handler(http);

    connection.run(handler).await?;

    Ok(())
}

struct Handler(Arc<Http>);

impl EventHandler for Handler {
    type Fut = Pin<Box<dyn Future<Output = ()> + Send>>;

    fn handle(&self, event: ServerToClientEvent) -> Self::Fut {
        Box::pin(handle_event(Arc::clone(&self.0), event))
    }
}

async fn handle_event(http: Arc<Http>, msg: ServerToClientEvent) {
    dbg!(&msg);
    match msg {
        ServerToClientEvent::Message { message } => {
            let _ = handle_message(http, message).await;
        }
        _ => {}
    }
}

async fn handle_message(http: Arc<Http>, message: Message) -> Result<(), Box<dyn std::error::Error>> {
    if message.content != "Hello" {
        return Ok(());
    }

    http.send_message(
        &message.channel,
        format!("Hello <@{}>", &message.author),
        rusty_ulid::generate_ulid_string(),
        vec![],
        vec![],
    )
    .await?;

    Ok(())
}
