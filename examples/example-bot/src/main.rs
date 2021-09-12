use std::collections::HashSet;
use std::convert::Infallible;
use std::future::Future;
use std::iter::FromIterator;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use robespierre::framework::standard::extractors::{
    Args, AuthorMember, RawArgs, RequiredServerPermissions, Rest,
};
use robespierre::framework::standard::{
    macros::command, AfterHandlerCodeFn, Command, CommandCodeFn, CommandResult, FwContext,
    NormalMessageHandlerCodeFn, StandardFramework,
};
use robespierre::model::mention::Mentionable;
use robespierre::model::ChannelIdExt;
use robespierre::{async_trait, model::MessageExt, Context, EventHandler, EventHandlerWrap};
use robespierre::{Authentication, CacheHttp, CacheWrap, FrameworkWrap, UserData};
use robespierre_cache::CacheConfig;
use robespierre_events::Connection;
use robespierre_http::Http;
use robespierre_models::autumn::AttachmentTag;
use robespierre_models::channels::{Channel, Message, MessageContent, ReplyData};
use robespierre_models::id::{ChannelId, ServerId, UserId};
use robespierre_models::servers::ServerPermissions;
use robespierre_models::users::User;

struct CommandCounter;
impl robespierre::typemap::Key for CommandCounter {
    type Value = Arc<AtomicUsize>;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let token = std::env::var("TOKEN")
        .expect("Cannot get token; set environment variable TOKEN=... and run again");
    let auth = Authentication::bot(token);

    let http = Http::new(&auth).await?;
    let connection = Connection::connect(&auth).await?;

    let typemap = {
        let mut typemap = robespierre::typemap::ShareMap::custom();
        typemap.insert::<CommandCounter>(Arc::new(AtomicUsize::new(0)));

        typemap
    };

    let ctx = Context::new(http, typemap).with_cache(CacheConfig::default());

    let owners = HashSet::from_iter(["01FE638VK54XZ6FEK167D4VC9N".parse().unwrap()]);

    let fw = StandardFramework::default()
        .configure(|c| c.prefix("!").owners(owners))
        .group(|g| {
            g.name("General")
                .command(|| Command::new("ping", ping as CommandCodeFn).alias("pong"))
                .command(|| Command::new("repeat", repeat as CommandCodeFn).owners_only(true))
                .command(|| Command::new("repeat2", repeat2 as CommandCodeFn).owners_only(true))
                .command(|| Command::new("repeat3", repeat3 as CommandCodeFn).owners_only(true))
                .command(|| Command::new("stat_user", stat_user as CommandCodeFn))
                .command(|| Command::new("stat_channel", stat_channel as CommandCodeFn))
                .command(|| Command::new("ban_perm_test", ban_perm_test as CommandCodeFn))
                .command(|| Command::new("requires_ban_perm", requires_ban_perm as CommandCodeFn))
                .command(|| Command::new("ulid_timestamp", ulid_timestamp as CommandCodeFn))
                .command(|| Command::new("arg_test", arg_test as CommandCodeFn).owners_only(true))
                .command(|| Command::new("say", say as CommandCodeFn).owners_only(true))
        })
        .normal_message(normal_message as NormalMessageHandlerCodeFn)
        .after(after_handler as AfterHandlerCodeFn);

    connection
        .run(
            ctx,
            CacheWrap::new(EventHandlerWrap::new(FrameworkWrap::new(fw, Handler))),
        )
        .await?;

    Ok(())
}

#[command]
async fn ping(ctx: &FwContext, message: &Message) -> CommandResult {
    let d = ctx.data_lock_write().await;
    let commands = d.get::<CommandCounter>().unwrap();
    let num = commands.fetch_add(1, Ordering::Relaxed);
    message.reply(ctx, "Who pinged me?!").await?;
    message
        .reply(ctx, format!("I got {} pings since I came online", num))
        .await?;

    Ok(())
}

#[command]
async fn repeat(ctx: &FwContext, message: &Message, arg: Args<(String,)>) -> CommandResult {
    let s = arg.0 .0;

    message.reply(ctx, s).await?;

    Ok(())
}

#[command]
async fn say(ctx: &FwContext, message: &Message, RawArgs(args): RawArgs) -> CommandResult {
    message.reply(ctx, &*args).await?;

    Ok(())
}

#[command]
async fn stat_channel(
    ctx: &FwContext,
    message: &Message,
    Args((channel,)): Args<(Channel,)>,
) -> CommandResult {
    message.reply(ctx, format!("{:?}", channel)).await?;

    Ok(())
}

#[command]
async fn stat_user(
    ctx: &FwContext,
    message: &Message,
    Args((user,)): Args<(User,)>,
) -> CommandResult {
    message.reply(ctx, format!("{:?}", user)).await?;

    Ok(())
}

#[command]
async fn ulid_timestamp(
    ctx: &FwContext,
    message: &Message,
    Args((ulid,)): Args<(rusty_ulid::Ulid,)>,
) -> CommandResult {
    message.reply(ctx, ulid.datetime().to_string()).await?;
    Ok(())
}

#[command]
async fn repeat2(
    ctx: &FwContext,
    message: &Message,
    #[delimiter(",")] Args((s1, s2)): Args<(String, String)>,
    member: Option<AuthorMember>,
) -> CommandResult {
    println!("{:?}", &member);

    message
        .reply(ctx, format!("first: {}, second: {}", s1, s2))
        .await?;

    Ok(())
}

#[command]
async fn repeat3(
    ctx: &FwContext,
    message: &Message,
    #[delimiter(",")] Args((s1, user, Rest(s2))): Args<(String, Option<UserId>, Rest<String>)>,
) -> CommandResult {
    println!("{:?}", &user);

    message
        .reply(
            ctx,
            format!("first: {}, user: {:?}\n, second: {}", s1, user, s2),
        )
        .await?;

    Ok(())
}

#[command]
async fn ban_perm_test(
    ctx: &FwContext,
    msg: &Message,
    AuthorMember(member): AuthorMember,
) -> CommandResult {
    // let channel = msg.channel(ctx).await?;
    let server = msg.server(ctx).await?.unwrap();

    let result = robespierre_models::permissions_utils::member_has_permissions(
        &member,
        ServerPermissions::BAN_MEMBERS,
        &server,
    );

    msg.reply(ctx, format!("{}", result)).await?;

    Ok(())
}

struct MyArgTy(String);

impl FromStr for MyArgTy {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

robespierre::from_str_arg_impl!(MyArgTy);

#[command]
async fn arg_test(
    ctx: &FwContext,
    message: &Message,
    Args((MyArgTy(arg),)): Args<(MyArgTy,)>,
) -> CommandResult {
    message.reply(ctx, arg).await?;

    Ok(())
}

#[command]
async fn requires_ban_perm(
    ctx: &FwContext,
    msg: &Message,
    _: RequiredServerPermissions<{ ServerPermissions::bits(&ServerPermissions::BAN_MEMBERS) }>,
) -> CommandResult {
    msg.reply(ctx, "If I got here, author has ban perms")
        .await?;

    Ok(())
}

async fn normal_message_impl(ctx: &FwContext, message: &Message) {
    if !matches!(&message.content, MessageContent::Content(s) if s == "Hello") {
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
            AttachmentTag::Attachments,
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
