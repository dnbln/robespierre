// src/main.rs
use robespierre::framework::standard::{macros::command, CommandResult, FwContext};
use robespierre::framework::standard::{Command, CommandCodeFn, StandardFramework};
use robespierre::model::MessageExt;
use robespierre::Authentication;
use robespierre::CacheWrap;
use robespierre::Context;
use robespierre::EventHandlerWrap;
use robespierre::FrameworkWrap;
use robespierre_cache::CacheConfig;
use robespierre_events::Connection;
use robespierre_http::Http;
use robespierre_models::channels::Message;

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

    let fw = StandardFramework::default()
        .configure(|c| c.prefix("!"))
        .group(|g| {
            g.name("General")
                .command(|| Command::new("ping", ping as CommandCodeFn))
        });
    let handler = FrameworkWrap::new(fw, Handler);
    let handler = CacheWrap::new(EventHandlerWrap::new(handler));
    connection.run(context, handler).await?;

    Ok(())
}

#[command]
async fn ping(ctx: &FwContext, msg: &Message) -> CommandResult {
    msg.reply(ctx, "pong").await?;
    Ok(())
}

#[derive(Clone)]
struct Handler;

#[robespierre::async_trait]
impl robespierre::EventHandler for Handler {}
