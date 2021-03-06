//! Example bot of using extractors
//!
//! Base: framework-example-bot
//! Book chapter: extractors.md

use robespierre::framework::standard::extractors::{Args, Author, RawArgs, Rest};
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
use robespierre_models::channels::{Channel, Message};
use robespierre_models::users::User;

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
                .command(|| Command::new("stat_user", stat_user as CommandCodeFn))
                .command(|| Command::new("stat_channel", stat_channel as CommandCodeFn))
                .command(|| Command::new("repeat", repeat as CommandCodeFn))
                .command(|| Command::new("repeat_with_commas", repeat_with_commas as CommandCodeFn))
                .command(|| {
                    Command::new(
                        "repeat_with_commas_using_rest",
                        repeat_with_commas_using_rest as CommandCodeFn,
                    )
                })
        });
    let handler = FrameworkWrap::new(fw, Handler);
    let handler = CacheWrap::new(EventHandlerWrap::new(handler));
    connection.run(context, handler).await?;

    Ok(())
}

#[command]
async fn stat_user(
    ctx: &FwContext,
    message: &Message,
    Args((user,)): Args<(User,)>, // parses a single argument as an UserId, and fetches the user with that id
) -> CommandResult {
    message.reply(ctx, format!("{:?}", user)).await?;

    Ok(())
}

#[command]
async fn stat_channel(
    ctx: &FwContext,
    message: &Message,
    Args((channel,)): Args<(Channel,)>, // parses a single argument as a ChannelId, and fetches the channel with that id
) -> CommandResult {
    message.reply(ctx, format!("{:?}", channel)).await?;

    Ok(())
}

#[command]
async fn repeat(
    ctx: &FwContext,
    message: &Message,
    Author(author): Author,
    RawArgs(args): RawArgs,
) -> CommandResult {
    if author.id != "<your user id>" {
        return Ok(());
    }

    // args is an Arc<String>
    message.reply(ctx, &*args).await?;

    Ok(())
}

#[command]
async fn repeat_with_commas(
    ctx: &FwContext,
    message: &Message,
    Author(author): Author,
    #[delimiter(",")] Args((arg1, arg2)): Args<(String, String)>,
) -> CommandResult {
    if author.id != "<your user id>" {
        return Ok(());
    }

    message
        .reply(ctx, format!("first: {}, second: {}", arg1, arg2))
        .await?;

    Ok(())
}

#[command]
async fn repeat_with_commas_using_rest(
    ctx: &FwContext,
    message: &Message,
    Author(author): Author,
    #[delimiter(",")] Args((arg1, Rest(arg2))): Args<(String, Rest<String>)>,
) -> CommandResult {
    if author.id != "<your user id>" {
        return Ok(());
    }

    message
        .reply(ctx, format!("first: {}, second: {}", arg1, arg2))
        .await?;

    Ok(())
}

#[derive(Clone)]
struct Handler;

#[robespierre::async_trait]
impl robespierre::EventHandler for Handler {}
