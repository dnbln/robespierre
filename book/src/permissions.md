# Note on permissions

There are 3 ways to check if an user / member has permissions to do something in a channel / server.

Let's say we want to implement a ban command:

## Manually checking permissions

```rust ,no_run
# use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
# use robespierre::framework::standard::extractors::{Args, AuthorMember, RawArgs, Rest};
# use robespierre::model::MessageExt;
# use robespierre_models::channels::Message;
# use robespierre_models::servers::ServerPermissions;
# use robespierre_models::users::User;

#[derive(thiserror::Error, Debug)]
#[error("missing perms")]
struct MissingPerms;

#[command]
async fn ban(
    ctx: &FwContext,
    msg: &Message,
    Args((user, Rest(reason))): Args<(User, Rest<Option<String>>)>,
    AuthorMember(member): AuthorMember,
) -> CommandResult {
    let server = msg.server(ctx).await?.unwrap();
    
    let result = robespierre_models::permissions_utils::member_has_permissions(
        &member,
        ServerPermissions::BAN_MEMBERS,
        &server,
    );

    if !result { return Err(MissingPerms.into()); }

    // ban
    Ok(())
}
```

If you need to check if an user has permissions in a given channel use `member_has_permissions_in_channel`,
and pass the `ChannelPermissions` you want to check + a reference to the channel you want to check for permissions in.

## Extractor version

```rust ,no_run
# use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
# use robespierre::framework::standard::extractors::{Args, AuthorMember, RawArgs, Rest, RequiredPermissions};
# use robespierre::model::MessageExt;
# use robespierre_models::channels::Message;
# use robespierre_models::channels::ChannelPermissions;
# use robespierre_models::servers::ServerPermissions;
# use robespierre_models::users::User;

#[command]
async fn ban(
    ctx: &FwContext,
    msg: &Message,
    Args((user, Rest(reason))): Args<(User, Rest<Option<String>>)>,
    _: RequiredPermissions<
        { ServerPermissions::bits(&ServerPermissions::BAN_MEMBERS) },
        { ChannelPermissions::bits(&ChannelPermissions::empty()) },
    >,
) -> CommandResult {
    // ban
    Ok(())
}
```

If one of them is `T::bits(&T::empty())`, it can be rewritten with the `RequiredServerPermissions`
and `RequiredChannelPermissions` utils.
In our example:

```rust ,no_run
# use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
# use robespierre::framework::standard::extractors::{Args, AuthorMember, RawArgs, Rest};
# use robespierre::model::MessageExt;
# use robespierre_models::channels::Message;
# use robespierre_models::servers::ServerPermissions;
# use robespierre_models::users::User;
# 
use robespierre::framework::standard::extractors::RequiredServerPermissions;

#[command]
async fn ban(
    ctx: &FwContext,
    msg: &Message,
    Args((user, Rest(reason))): Args<(User, Rest<Option<String>>)>,
    _: RequiredServerPermissions<{ ServerPermissions::bits(&ServerPermissions::BAN_MEMBERS) }>,
) -> CommandResult {
#     // ban
#     Ok(())
}
```

## Adding required permissions to the command when creating the framework

```rust ,no_run
# use robespierre::CacheWrap;
# use robespierre::EventHandlerWrap;
# use robespierre::Context;
# use robespierre::Authentication;
# use robespierre::model::MessageExt;
# use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
# use robespierre::framework::standard::{StandardFramework, Command, CommandCodeFn};
# use robespierre::framework::standard::extractors::{Args, Rest};
# use robespierre::FrameworkWrap;
# use robespierre_cache::CacheConfig;
# use robespierre_events::Connection;
# use robespierre_http::Http;
# use robespierre_models::channels::Message;
use robespierre_models::servers::ServerPermissions;
# use robespierre_models::users::User;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
#     tracing_subscriber::fmt::init();
# 
#     let token = std::env::var("TOKEN")
#         .expect("Cannot get token; set environment variable TOKEN=... and run again");
# 
#     let auth = Authentication::bot(token);
# 
#     let http = Http::new(&auth).await?;
#     let connection = Connection::connect(&auth).await?;
# 
#     let context = Context::new(http, robespierre::typemap::ShareMap::custom()).with_cache(CacheConfig::default());

    let fw = StandardFramework::default()
        .configure(|c| c.prefix("!"))
        .group(|g| {
            g.name("General")
                .command(|| Command::new("ban", ban as CommandCodeFn).required_server_permissions(ServerPermissions::BAN_MEMBERS))
        });

#     let handler = FrameworkWrap::new(fw, Handler);
#     let handler = CacheWrap::new(EventHandlerWrap::new(handler));
# 
#     connection.run(context, handler).await?;
# 
#     Ok(())
}

#[command]
async fn ban(
    ctx: &FwContext,
    msg: &Message,
    Args((user, Rest(reason))): Args<(User, Rest<Option<String>>)>,
) -> CommandResult {
    // ban
    Ok(())
}

# #[derive(Clone)]
# struct Handler;
# 
# #[robespierre::async_trait]
# impl robespierre::EventHandler for Handler {}
```
