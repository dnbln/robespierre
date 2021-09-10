# User data

Let's start with the code from [the previous chapter](framework.md):
```rust ,no_run
// src/main.rs
use robespierre::CacheWrap;
use robespierre::EventHandlerWrap;
use robespierre::Context;
use robespierre::Authentication;
use robespierre::model::MessageExt;
use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
use robespierre::framework::standard::{StandardFramework, Command, CommandCodeFn};
use robespierre::FrameworkWrap;
use robespierre_cache::CacheConfig;
use robespierre_events::Connection;
use robespierre_http::Http;
use robespierre_models::channel::Message;

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
```

Specifically this line is of interest:

```rust ,no_run
// src/main.rs
# use robespierre::CacheWrap;
# use robespierre::EventHandlerWrap;
# use robespierre::Context;
# use robespierre::Authentication;
# use robespierre::model::MessageExt;
# use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
# use robespierre::framework::standard::{StandardFramework, Command, CommandCodeFn};
# use robespierre::FrameworkWrap;
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

    let context = Context::new(http, robespierre::typemap::ShareMap::custom()).with_cache(CacheConfig::default());

#     let fw = StandardFramework::default()
#         .configure(|c| c.prefix("!"))
#         .group(|g| {
#             g.name("General")
#                 .command(|| Command::new("ping", ping as CommandCodeFn))
#         });
# 
#     let handler = FrameworkWrap::new(fw, Handler);
#     let handler = CacheWrap::new(EventHandlerWrap::new(handler));
# 
#     connection.run(context, handler).await?;
# 
#     Ok(())
}

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

So let's add a "command counter", just a number
to count how many commands we executed.

For performance, instead of using an
`Arc<Mutex<usize>>`, we'll use an `Arc<AtomicUsize>`.

First, we'll start with a definition of a typemap key:

```rust ,no_run
// src/main.rs
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
# use robespierre::CacheWrap;
# use robespierre::EventHandlerWrap;
# use robespierre::Context;
# use robespierre::Authentication;
# use robespierre::model::MessageExt;
# use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
# use robespierre::framework::standard::{StandardFramework, Command, CommandCodeFn};
# use robespierre::FrameworkWrap;
# use robespierre_cache::CacheConfig;
# use robespierre_events::Connection;
# use robespierre_http::Http;
# use robespierre_models::channel::Message;

struct CommandCounterKey;

impl robespierre::typemap::Key for CommandCounterKey {
    type Value = Arc<AtomicUsize>;
}

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
#     let fw = StandardFramework::default()
#         .configure(|c| c.prefix("!"))
#         .group(|g| {
#             g.name("General")
#                 .command(|| Command::new("ping", ping as CommandCodeFn))
#         });
# 
#     let handler = FrameworkWrap::new(fw, Handler);
#     let handler = CacheWrap::new(EventHandlerWrap::new(handler));
# 
#     connection.run(context, handler).await?;
# 
#     Ok(())
# }
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

And add it to the map:

```rust ,no_run
// src/main.rs
# use std::sync::Arc;
# use std::sync::atomic::AtomicUsize;
# use robespierre::CacheWrap;
# use robespierre::EventHandlerWrap;
# use robespierre::Context;
# use robespierre::Authentication;
# use robespierre::model::MessageExt;
# use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
# use robespierre::framework::standard::{StandardFramework, Command, CommandCodeFn};
# use robespierre::FrameworkWrap;
# use robespierre_cache::CacheConfig;
# use robespierre_events::Connection;
# use robespierre_http::Http;
# use robespierre_models::channel::Message;

# struct CommandCounterKey;
# 
# impl robespierre::typemap::Key for CommandCounterKey {
#     type Value = Arc<AtomicUsize>;
# }

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

    let mut data = robespierre::typemap::ShareMap::custom();
    data.insert::<CommandCounterKey>(Arc::new(AtomicUsize::new(0)));
    let context = Context::new(http, data).with_cache(CacheConfig::default());

#     let fw = StandardFramework::default()
#         .configure(|c| c.prefix("!"))
#         .group(|g| {
#             g.name("General")
#                 .command(|| Command::new("ping", ping as CommandCodeFn))
#         });
# 
#     let handler = FrameworkWrap::new(fw, Handler);
#     let handler = CacheWrap::new(EventHandlerWrap::new(handler));
# 
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

And when we get a command we have to increment it:

```rust ,no_run
// src/main.rs
# use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
# use robespierre::CacheWrap;
# use robespierre::EventHandlerWrap;
# use robespierre::Context;
# use robespierre::Authentication;
use robespierre::UserData;
# use robespierre::model::MessageExt;
# use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
# use robespierre::framework::standard::{StandardFramework, Command, CommandCodeFn};
# use robespierre::FrameworkWrap;
# use robespierre_cache::CacheConfig;
# use robespierre_events::Connection;
# use robespierre_http::Http;
# use robespierre_models::channel::Message;

# struct CommandCounterKey;
# 
# impl robespierre::typemap::Key for CommandCounterKey {
#     type Value = Arc<AtomicUsize>;
# }
# 
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
#     let mut data = robespierre::typemap::ShareMap::custom();
#     data.insert::<CommandCounterKey>(Arc::new(AtomicUsize::new(0)));
#     let context = Context::new(http, data).with_cache(CacheConfig::default());
# 
#     let fw = StandardFramework::default()
#         .configure(|c| c.prefix("!"))
#         .group(|g| {
#             g.name("General")
#                 .command(|| Command::new("ping", ping as CommandCodeFn))
#         });
# 
#     let handler = FrameworkWrap::new(fw, Handler);
#     let handler = CacheWrap::new(EventHandlerWrap::new(handler));
# 
#     connection.run(context, handler).await?;
# 
#     Ok(())
# }

#[command]
async fn ping(ctx: &FwContext, msg: &Message) -> CommandResult {
#     msg.reply(ctx, "pong").await?;

    let data = ctx.data_lock_read().await;
    let counter = data.get::<CommandCounterKey>().unwrap();
    counter.fetch_add(1, Ordering::SeqCst);
    
#     Ok(())
}

# #[derive(Clone)]
# struct Handler;
# 
# #[robespierre::async_trait]
# impl robespierre::EventHandler for Handler {}
```

And finally, let's add another command to display the counter:

```rust ,no_run
// src/main.rs
# use std::sync::Arc;
# use std::sync::atomic::{AtomicUsize, Ordering};
# use robespierre::CacheWrap;
# use robespierre::EventHandlerWrap;
# use robespierre::Context;
# use robespierre::Authentication;
# use robespierre::UserData;
# use robespierre::model::MessageExt;
# use robespierre::framework::standard::{FwContext, CommandResult, macros::command};
# use robespierre::framework::standard::{StandardFramework, Command, CommandCodeFn};
# use robespierre::FrameworkWrap;
# use robespierre_cache::CacheConfig;
# use robespierre_events::Connection;
# use robespierre_http::Http;
# use robespierre_models::channel::Message;

# struct CommandCounterKey;
# 
# impl robespierre::typemap::Key for CommandCounterKey {
#     type Value = Arc<AtomicUsize>;
# }

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
#     let mut data = robespierre::typemap::ShareMap::custom();
#     data.insert::<CommandCounterKey>(Arc::new(AtomicUsize::new(0)));
#     let context = Context::new(http, data).with_cache(CacheConfig::default());

    let fw = StandardFramework::default()
        .configure(|c| c.prefix("!"))
        .group(|g| {
            g.name("General")
                .command(|| Command::new("ping", ping as CommandCodeFn))
                .command(|| Command::new("command_counter", command_counter as CommandCodeFn))
        });

#     let handler = FrameworkWrap::new(fw, Handler);
#     let handler = CacheWrap::new(EventHandlerWrap::new(handler));
# 
#     connection.run(context, handler).await?;
# 
#     Ok(())
}

# #[command]
# async fn ping(ctx: &FwContext, msg: &Message) -> CommandResult {
#     msg.reply(ctx, "pong").await?;
# 
#     let data = ctx.data_lock_read().await;
#     let counter = data.get::<CommandCounterKey>().unwrap();
#     counter.fetch_add(1, Ordering::SeqCst);
#     
#     Ok(())
# }

#[command]
async fn command_counter(ctx: &FwContext, msg: &Message) -> CommandResult {
    let data = ctx.data_lock_read().await;
    let counter = data.get::<CommandCounterKey>().unwrap();
    let count = counter.fetch_add(1, Ordering::SeqCst); // this is itself a command,
                                                        // so fetch previous count and add one.
    msg.reply(ctx, format!("I received {} commands since I started running", count)).await?;
    
    Ok(())
}

# #[derive(Clone)]
# struct Handler;
# 
# #[robespierre::async_trait]
# impl robespierre::EventHandler for Handler {}
```

For the source code of this, see the [framework-with-data-example](https://github.com/dblanovschi/robespierre/tree/main/examples/framework-with-data-example).