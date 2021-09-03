use std::future::Future;
use std::pin::Pin;

use robespierre::framework::standard::{AfterHandlerCodeFn, Command, CommandCodeFn, CommandResult, FwContext, NormalMessageHandlerCodeFn, StandardFramework, command};
use robespierre::model::mention::Mentionable;
use robespierre::model::ChannelIdExt;
use robespierre::{async_trait, model::MessageExt, Context, EventHandler, EventHandlerWrap};
use robespierre::{Authentication, CacheHttp, CacheWrap, FrameworkWrap};
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

    let fw = StandardFramework::default()
        .configure(|c| c.prefix("!"))
        .group(|g| {
            g.name("General")
                .command(|| Command::new("ping", ping as CommandCodeFn))
        })
        .normal_message(normal_message as NormalMessageHandlerCodeFn)
        .after(after_handler as AfterHandlerCodeFn);

    connection
        .run(
            ctx,
            EventHandlerWrap::new(CacheWrap::new(FrameworkWrap::new(fw, Handler))),
        )
        .await?;

    Ok(())
}

#[command]
async fn ping(
    ctx: &FwContext,
    message: &Message,
    _args: &str,
) -> CommandResult {
    message.reply(ctx, "Who pinged me?!").await?;

    Ok(())
}

async fn normal_message_impl(ctx: &FwContext, message: &Message) {
    if message.content != "Hello" {
        return;
    }

    let author = message.author(ctx).await.unwrap();
    let channel = message.channel(ctx).await.unwrap();
    let server = message.server(ctx).await.unwrap();

    let _session = message.channel.start_typing(ctx);
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let att_id = ctx
        .http()
        .upload_autumn(
            AutumnTag::Attachments,
            "help".to_string(),
            "help me".to_string().into_bytes(),
        )
        .await
        .unwrap();

    let _ = message
        .channel
        .send_message(ctx, |msg| {
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

fn normal_message<'a>(
    ctx: &'a FwContext,
    message: &'a Message,
) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
    Box::pin(normal_message_impl(ctx, message))
}

async fn after_handler_impl<'a>(_ctx: &'a FwContext, _message: &'a Message, result: CommandResult) {
    match result {
        Ok(()) => {
            tracing::info!("Got ok!");
        }
        Err(err) => {
            tracing::error!("Error: {}; full: {:?}", err, err);
        }
    }
}

fn after_handler<'a>(
    ctx: &'a FwContext,
    message: &'a Message,
    result: CommandResult,
) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
    Box::pin(async move { after_handler_impl(ctx, message, result).await })
}

#[derive(Copy, Clone)]
struct Handler;

#[async_trait]
impl EventHandler for Handler {
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
