# Framework
Sometimes, matching against a lot of commands
in the message handler can be cumbersome, and
we just want to call functions for each command.

The standard framework can help us a lot then:

Let's take our example from where we left off in
[the first chapter](writing-an-example-bot.md):

```rust
// src/main.rs
use robespierre::CacheWrap;
use robespierre::EventHandlerWrap;
use robespierre::Context;
use robespierre::Authentication;
use robespierre::model::MessageExt;
use robespierre_cache::CacheConfig;
use robespierre_events::Connection;
use robespierre_http::Http;
use robespierre_models::channel::Message;
use robespierre_models::channel::MessageContent;
use robespierre_models::events::ReadyEvent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let token = std::env::var("TOKEN")
        .expect("Cannot get token; set environment variable TOKEN=... and run again");

    let auth = Authentication::bot(token);
    
    let http = Http::new(&auth).await?;
    let connection = Connection::connect(&auth).await?;
    
    let context = Context::new(http, robespierre::typemap::ShareMap::custom()).with_cache(CacheConfig::default());
    
    let handler = Handler;
    let handler = CacheWrap::new(EventHandlerWrap::new(handler));
    
    connection.run(context, handler).await?;

    Ok(())
}


#[derive(Clone)]
struct Handler;

#[robespierre::async_trait]
impl robespierre::EventHandler for Handler {
    async fn on_ready(&self, ctx: Context, ready: ReadyEvent) {
        tracing::info!("We're ready!");
    }

    async fn on_message(&self, ctx: Context, message: Message) {
        if matches!(&message.content, MessageContent::Content(s) if s == "ping") {
            let _ = message.reply(&ctx, "pong").await;
        }
    }
}
```

Now, let's get rid of the `on_ready` and `on_message` functions:

```rust
// src/main.rs
# use robespierre::CacheWrap;
# use robespierre::EventHandlerWrap;
# use robespierre::Context;
# use robespierre::Authentication;
# use robespierre::model::MessageExt;
# use robespierre_cache::CacheConfig;
# use robespierre_events::Connection;
# use robespierre_http::Http;
# use robespierre_models::channel::Message;

# #[tokio::main]
# async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
#     let handler = Handler;
#     let handler = CacheWrap::new(EventHandlerWrap::new(handler));
#     connection.run(context, handler).await?;
# 
#     Ok(())
# }
# 
# 
# #[derive(Clone)]
# struct Handler;

#[robespierre::async_trait]
impl robespierre::EventHandler for Handler {}
```

The signature of an event handler is defined as the `robespierre::framework::standard::CommandCodeFn` type, and looks like this:
```rust,ignore
fn command<'a>(
    ctx: &'a FwContext, // similar to Context
    msg: &'a Message,
    args: &'a str,
) -> Pin<Box<dyn Future<Output=CommandResult> + Send + 'a>>;
```

To hide the nastier implementation details, the `robespierre::framework::standard::macros::command` macro helps a little, and turns this:
```rust,ignore
#[command]
async fn command(
    ctx: &FwContext,
    msg: &Message,
    args: &str,
) -> CommandResult {
    Ok(())
}
```

Into a function that can be given where `robespierre::framework::standard::CommandCodeFn` is expected.

First, let's start with a `ping` command:
```rust
// src/main.rs
# use robespierre::CacheWrap;
# use robespierre::EventHandlerWrap;
# use robespierre::Context;
# use robespierre::Authentication;
# use robespierre::model::MessageExt;
use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
# use robespierre_cache::CacheConfig;
# use robespierre_events::Connection;
# use robespierre_http::Http;
# use robespierre_models::channel::Message;

# #[tokio::main]
# async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
# 
#     let handler = Handler;
#     let handler = CacheWrap::new(EventHandlerWrap::new(handler));
#     connection.run(context, handler).await?;
# 
#     Ok(())
# }

#[command]
async fn ping(ctx: &FwContext, msg: &Message) -> CommandResult {
    msg.reply(ctx, "pong").await?;
    Ok(())
}
# 
# #[derive(Clone)]
# struct Handler;
# 
# #[robespierre::async_trait]
# impl robespierre::EventHandler for Handler {}
```

Now, let's go build a `StandardFramework`:

```rust
// src/main.rs
# use robespierre::CacheWrap;
# use robespierre::EventHandlerWrap;
# use robespierre::Context;
# use robespierre::Authentication;
# use robespierre::model::MessageExt;
# use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
use robespierre::framework::standard::{StandardFramework, Command, CommandCodeFn};
# use robespierre_cache::CacheConfig;
# use robespierre_events::Connection;
# use robespierre_http::Http;
# use robespierre_models::channel::Message;

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
                .command(|| Command::new("ping", ping as CommandCodeFn))
        });

#     let handler = Handler;
#     let handler = CacheWrap::new(EventHandlerWrap::new(handler));
#     connection.run(context, handler).await?;
# 
#     Ok(())
}
# 
# #[command]
# async fn ping(ctx: &FwContext, msg: &Message) -> CommandResult {
#     msg.reply(ctx, "pong").await?;
#     Ok(())
# }
# 
# #[derive(Clone)]
# struct Handler;
# 
# #[robespierre::async_trait]
# impl robespierre::EventHandler for Handler {}
```

And now let's also use it:

```rust
// src/main.rs
# use robespierre::CacheWrap;
# use robespierre::EventHandlerWrap;
# use robespierre::Context;
# use robespierre::Authentication;
# use robespierre::model::MessageExt;
# use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
# use robespierre::framework::standard::{StandardFramework, Command, CommandCodeFn};
use robespierre::FrameworkWrap;
# use robespierre_cache::CacheConfig;
# use robespierre_events::Connection;
# use robespierre_http::Http;
# use robespierre_models::channel::Message;

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
# 
#     let fw = StandardFramework::default()
#         .configure(|c| c.prefix("!"))
#         .group(|g| {
#             g.name("General")
#                 .command(|| Command::new("ping", ping as CommandCodeFn))
#         });

    let handler = FrameworkWrap::new(fw, Handler);
    let handler = CacheWrap::new(EventHandlerWrap::new(handler));

#     connection.run(context, handler).await?;
# 
#     Ok(())
}
# 
# #[command]
# async fn ping(ctx: &FwContext, msg: &Message) -> CommandResult {
#     msg.reply(ctx, "pong").await?;
#     Ok(())
# }
# 
# #[derive(Clone)]
# struct Handler;
# 
# #[robespierre::async_trait]
# impl robespierre::EventHandler for Handler {}
```

Then run:
```bash
TOKEN=... cargo run
```

And send "!ping" in a channel where the bot can see it.
If everything went correctly, the bot should reply with "pong" to your message.

For the source code of this, see the [the framework-example-bot example](https://github.com/dblanovschi/robespierre/tree/main/examples/framework-example-bot).