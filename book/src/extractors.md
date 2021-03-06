# Extractors

Extractors here work like they do in `actix_web`.
They implement the `FromMessage` trait, and get
the data they need from the message + context.

Note: The first 2 arguments are *always* the context
and the message; all the others are expected to be
extractors.

Note 2: They are only available when using the standard
framework.

Here's an example of a stat user command, that just
formats the user with the debug formatter and replies
with the result:

```rust ,no_run
# use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
# use robespierre::framework::standard::extractors::{Args, Author, RawArgs, Rest};
# use robespierre::model::MessageExt;
# use robespierre_models::channels::Message;
# use robespierre_models::users::User;

#[command]
async fn stat_user(ctx: &FwContext, message: &Message,
    Args((user,)): Args<(User,)> // parses a single argument as an UserId, and fetches the user with that id
) -> CommandResult {
    message.reply(ctx, format!("{:?}", user)).await?;

    Ok(())
}
```

Or a stat channel command:

```rust ,no_run
# use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
# use robespierre::framework::standard::extractors::{Args, Author, RawArgs, Rest};
# use robespierre::model::MessageExt;
# use robespierre_models::channels::{Message, Channel};

#[command]
async fn stat_channel(ctx: &FwContext, message: &Message,
    Args((channel,)): Args<(Channel,)> // parses a single argument as a ChannelId, and fetches the channel with that id
) -> CommandResult {
    message.reply(ctx, format!("{:?}", channel)).await?;

    Ok(())
}
```

Or a "repeat" command, which just echoes back the arguments
(be careful who you let to run this command):

```rust ,no_run
# use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
# use robespierre::framework::standard::extractors::{Args, Author, RawArgs, Rest};
# use robespierre::model::MessageExt;
# use robespierre_models::channels::Message;

#[command]
async fn repeat(ctx: &FwContext, message: &Message, Author(author): Author, RawArgs(args): RawArgs) -> CommandResult {
    if author.id != "<your user id>" {
        return Ok(());
    }

    message.reply(ctx, &*args).await?;

    Ok(())
}
```

They get added to the framework in the exact same way.

By default, the delimiter `Args` uses is ``, but you can change it like:

```rust ,no_run
# use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
# use robespierre::framework::standard::extractors::{Args, Author, RawArgs, Rest};
# use robespierre::model::MessageExt;
# use robespierre_models::channels::Message;

#[command]
async fn repeat_with_spaces(
    ctx: &FwContext,
    message: &Message,
    Author(author): Author,
    #[delimiter(" ")] // it's the default, can be removed
    Args((arg1, arg2)): Args<(String, String)>
) -> CommandResult {
    if author.id != "<your user id>" {
        return Ok(());
    }

    message.reply(ctx, format!("first: {}, second: {}", arg1, arg2)).await?;

    Ok(())
}

#[command]
async fn repeat_with_spaces_and_tabs(
    ctx: &FwContext,
    message: &Message,
    Author(author): Author,
    #[delimiters(" ", "\t")]
    Args((arg1, arg2)): Args<(String, String)>
) -> CommandResult {
    if author.id != "<your user id>" {
        return Ok(());
    }

    message.reply(ctx, format!("first: {}, second: {}", arg1, arg2)).await?;

    Ok(())
}

#[command]
async fn repeat_with_spaces_and_tabs_2(
    ctx: &FwContext,
    message: &Message,
    Author(author): Author,
    #[delimiter(" ")]
    #[delimiter("\t")] // they cumulate
    Args((arg1, arg2)): Args<(String, String)>
) -> CommandResult {
    if author.id != "<your user id>" {
        return Ok(());
    }

    message.reply(ctx, format!("first: {}, second: {}", arg1, arg2)).await?;

    Ok(())
}

#[command]
async fn repeat_with_commas(
    ctx: &FwContext,
    message: &Message,
    Author(author): Author,
    #[delimiter(",")]
    Args((arg1, arg2)): Args<(String, String)>
) -> CommandResult {
    if author.id != "<your user id>" {
        return Ok(());
    }

    message.reply(ctx, format!("first: {}, second: {}", arg1, arg2)).await?;

    Ok(())
}
```

Note: valid values include everything that implements `Into<Cow<'static, str>>`,
but for `#[delimiters()]`, they have to all be of the same type.

If you need values of multiple types, then you can use multiple `#[delimiters]` attributes.

## Special types

There are 2 kinds of special argument types (omitting `.to_string()` calls for simplicity):

### `Option<T> where T: Arg`

Means "try to parse the argument as T, and if failed, pass None as arg, and continue trying to parse as if there was no arg here".

E.g. a type like:

```rust ,ignore
# fn f(
#[delimiter(",")]
args: Args<(String, Option<UserId>, String)>,
# )->{}
```

If given `aaa, bbb`, it will result `Args(("aaa", None, "bbb"))`.

While if given `aaa, <@AAAAAAAAAAAAAAAAAAAAAAAAAA>, bbb`, it will result `Args(("aaa", Some(...), "bbb"))`.

### `Rest<T> where T: Arg`

It means "use all the remaining text" to parse `T`.

E.g. a type like:

```rust ,ignore
# fn f(
#[delimiter(",")]
args: Args<(String, Rest<String>)>
# )->{}
```

Given:

```text
"aaa, bbb" => Args(("aaa", Rest("bbb")))
"aaa, bbb, ccc" => Args(("aaa", Rest("bbb, ccc")))
```

Note: No argument should ever come after a `Rest<T>`.

A full working example:

```rust ,no_run
use robespierre::CacheWrap;
use robespierre::EventHandlerWrap;
use robespierre::Context;
use robespierre::Authentication;
use robespierre::model::MessageExt;
use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
use robespierre::framework::standard::{StandardFramework, Command, CommandCodeFn};
use robespierre::framework::standard::extractors::{Args, Author, RawArgs, Rest};
use robespierre::FrameworkWrap;
use robespierre_cache::CacheConfig;
use robespierre_events::Connection;
use robespierre_http::Http;
use robespierre_models::users::User;
use robespierre_models::channels::{Channel, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let token = std::env::var("TOKEN")
        .expect("Cannot get token; set environment variable TOKEN=... and run again");

    let auth = Authentication::bot(token);

    let http = Http::new(&auth).await?;
    let connection = Connection::connect(&auth).await?;

    let context = Context::new(http, robespierre::typemap::ShareMap::custom()).with_cache(CacheConfig::default());

    let fw = StandardFramework::default()
        .configure(|c| c.prefix("!"))
        .group(|g| {
            g.name("General")
                .command(|| Command::new("stat_user", stat_user as CommandCodeFn))
                .command(|| Command::new("stat_channel", stat_channel as CommandCodeFn))
                .command(|| Command::new("repeat", repeat as CommandCodeFn))
                .command(|| Command::new("repeat_with_commas", repeat_with_commas as CommandCodeFn))
                .command(|| Command::new("repeat_with_commas_using_rest", repeat_with_commas_using_rest as CommandCodeFn))
        });
    let handler = FrameworkWrap::new(fw, Handler);
    let handler = CacheWrap::new(EventHandlerWrap::new(handler));
    connection.run(context, handler).await?;

    Ok(())
}

#[command]
async fn stat_user(ctx: &FwContext, message: &Message,
    Args((user,)): Args<(User,)> // parses a single argument as an UserId, and fetches the user with that id
) -> CommandResult {
    message.reply(ctx, format!("{:?}", user)).await?;

    Ok(())
}

#[command]
async fn stat_channel(ctx: &FwContext, message: &Message,
    Args((channel,)): Args<(Channel,)> // parses a single argument as a ChannelId, and fetches the channel with that id
) -> CommandResult {
    message.reply(ctx, format!("{:?}", channel)).await?;

    Ok(())
}

#[command]
async fn repeat(ctx: &FwContext, message: &Message, Author(author): Author, RawArgs(args): RawArgs) -> CommandResult {
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
    #[delimiter(",")]
    Args((arg1, arg2)): Args<(String, String)>
) -> CommandResult {
    if author.id != "<your user id>" {
        return Ok(());
    }

    message.reply(ctx, format!("first: {}, second: {}", arg1, arg2)).await?;

    Ok(())
}

#[command]
async fn repeat_with_commas_using_rest(
    ctx: &FwContext,
    message: &Message,
    Author(author): Author,
    #[delimiter(",")]
    Args((arg1, Rest(arg2))): Args<(String, Rest<String>)>
) -> CommandResult {
    if author.id != "<your user id>" {
        return Ok(());
    }

    message.reply(ctx, format!("first: {}, second: {}", arg1, arg2)).await?;

    Ok(())
}

#[derive(Clone)]
struct Handler;

#[robespierre::async_trait]
impl robespierre::EventHandler for Handler {}
```

As always, you can find the example [in the repo](https://github.com/dblanovschi/robespierre/tree/main/examples/extractors-example)
