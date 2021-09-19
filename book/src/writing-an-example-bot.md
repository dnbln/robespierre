# Writing an example bot

Create a new project:

```bash
cargo new --bin mybot
```

Add `robespierre`, `robespierre-cache`, `robespierre-http`, `robespierre-events`, and `robespierre-models`:

```toml
[dependencies]
robespierre = { version = "0.3.0", features = ["cache", "events", "framework", "framework-macros"] }
robespierre-cache = "0.3.0"
robespierre-http = "0.3.0"
robespierre-events = "0.3.0"
robespierre-models = "0.3.0"
```

We'll also need `tokio`, and if you want logging, `tracing`:

```toml
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.2" # not necessarely this but some global subscriber is required for logging
```

```rust ,no_run
// src/main.rs
fn main() {
    println!("Hello, world!");
}
```

Let's start with a stub main, running on tokio:

```rust ,no_run
// src/main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
```

Optional: initialize a global `tracing` subscriber:

```rust ,no_run
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    Ok(())
}
```

Now, get the bot token:

```rust ,no_run
// src/main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
 
    let token = std::env::var("TOKEN")
        .expect("Cannot get token; set environment variable TOKEN=... and run again");

    Ok(())
}
```

Create an authentication:

```rust ,no_run
// src/main.rs
use robespierre::Authentication;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
#     tracing_subscriber::fmt::init();

    let token = std::env::var("TOKEN")
        .expect("Cannot get token; set environment variable TOKEN=... and run again");

    let auth = Authentication::bot(token);

#     Ok(())
}
```

Then a http client:

```rust ,no_run
// src/main.rs
# use robespierre::Authentication;
use robespierre_http::Http;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
#     tracing_subscriber::fmt::init();

    let token = std::env::var("TOKEN")
        .expect("Cannot get token; set environment variable TOKEN=... and run again");

    let auth = Authentication::bot(token);

    let http = Http::new(&auth).await?;

#     Ok(())
}
```

And a websocket connection:

```rust ,no_run
// src/main.rs
# use robespierre::Authentication;
use robespierre_events::Connection;
# use robespierre_http::Http;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
#     tracing_subscriber::fmt::init();

    let token = std::env::var("TOKEN")
        .expect("Cannot get token; set environment variable TOKEN=... and run again");

    let auth = Authentication::bot(token);

    let http = Http::new(&auth).await?;

    let connection = Connection::connect(&auth).await?;

#     Ok(())
}
```

Now let's write a basic event handler (similar to how they work in serenity):

```rust ,no_run
// src/main.rs
use robespierre::Context;
# use robespierre::Authentication;
use robespierre_events::Connection;
# use robespierre_http::Http;
use robespierre_models::events::ReadyEvent;

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
# 
#     let connection = Connection::connect(&auth).await?;
#
#     Ok(())
# }

#[derive(Clone)]
struct Handler;

#[robespierre::async_trait]
impl robespierre::EventHandler for Handler {
    async fn on_ready(&self, ctx: Context, ready: ReadyEvent) {
        tracing::info!("We're ready!");
    }
}
```

And create the context and handler:

```rust ,no_run
// src/main.rs
# use robespierre::Context;
# use robespierre::Authentication;
use robespierre_cache::CacheConfig;
# use robespierre_events::Connection;
# use robespierre_http::Http;
# use robespierre_models::events::ReadyEvent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
#     tracing_subscriber::fmt::init();
# 
#     let token = std::env::var("TOKEN")
#         .expect("Cannot get token; set environment variable TOKEN=... and run again");
# 
#     let auth = Authentication::bot(token);

    let http = Http::new(&auth).await?;

    let connection = Connection::connect(&auth).await?;

    let context = Context::new(http, robespierre::typemap::ShareMap::custom()); // (*)
    // and if you want a cache:
    let context = context.with_cache(CacheConfig::default());
    let handler = Handler;

    Ok(())
}

# #[derive(Clone)]
# struct Handler;
# 
# #[robespierre::async_trait]
# impl robespierre::EventHandler for Handler {
#     async fn on_ready(&self, ctx: Context, ready: ReadyEvent) {
#         tracing::info!("We're ready!");
#     }
# }
```

*) More on this in [the user data chapter](user-data.md), for now just leave it as it is.

Now let's "run" the connection, listening to events and handling them:

```rust ,no_run
// src/main.rs
use robespierre::EventHandlerWrap;
# use robespierre::Context;
# use robespierre::Authentication;
# use robespierre_cache::CacheConfig;
# use robespierre_events::Connection;
# use robespierre_http::Http;
# use robespierre_models::events::ReadyEvent;

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
# 
#     let connection = Connection::connect(&auth).await?;

    let context = Context::new(http, robespierre::typemap::ShareMap::custom()).with_cache(CacheConfig::default());

    let handler = Handler;

    let handler = EventHandlerWrap::new(handler); // explained in a bit
    connection.run(context, handler).await?;

    Ok(())
}

# #[derive(Clone)]
# struct Handler;
# 
# #[robespierre::async_trait]
# impl robespierre::EventHandler for Handler {
#     async fn on_ready(&self, ctx: Context, ready: ReadyEvent) {
#         tracing::info!("We're ready!");
#     }
# }
```

First, we have to make a distinction between the 2 types of handlers availabe:

- `T: EventHandler`
- `T: RawEventHandler`

The `EventHandler` contains functions like `on_ready`, `on_message`, `on_message_update`,
while the `RawEventHandler` only contains the function `async fn handle(self, context, event)`,
which is supposed to handle any type of event the server sends.

Now, `Connection::run` takes a `RawEventHandler`, but our `Handler` is an `EventHandler`, not a `RawEventHandler`.
The `EventHandlerWrap` is an utility to convert the events from a `RawEventHandler` to call the functions of an `EventHandler`.
It itself is a `RawEventHandler`.

Now, since we used a cache in the context, we also have to update it when we receive events.
That can be done by using another utility, `robespierre::CacheWrap`.

`robespierre::CacheWrap` is a `RawEventHandler`, and also forwards the events to another `RawEventHandler` after it updates the cache.

To use it:

```rust ,no_run
// src/main.rs
use robespierre::CacheWrap;
# use robespierre::EventHandlerWrap;
# use robespierre::Context;
# use robespierre::Authentication;
# use robespierre_cache::CacheConfig;
# use robespierre_events::Connection;
# use robespierre_http::Http;
# use robespierre_models::events::ReadyEvent;

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
# 
#     let connection = Connection::connect(&auth).await?;

    let context = Context::new(http, robespierre::typemap::ShareMap::custom()).with_cache(CacheConfig::default());

    let handler = Handler;

    let handler = CacheWrap::new(EventHandlerWrap::new(handler));
    connection.run(context, handler).await?;

    Ok(())
}

# #[derive(Clone)]
# struct Handler;
# 
# #[robespierre::async_trait]
# impl robespierre::EventHandler for Handler {
#     async fn on_ready(&self, ctx: Context, ready: ReadyEvent) {
#         tracing::info!("We're ready!");
#     }
# }
```

Now to finish, let's make the bot reply with "pong" whenever someone says "ping":

```rust ,no_run
// src/main.rs
# use robespierre::CacheWrap;
# use robespierre::EventHandlerWrap;
# use robespierre::Context;
# use robespierre::Authentication;
use robespierre::model::MessageExt;
# use robespierre_cache::CacheConfig;
# use robespierre_events::Connection;
# use robespierre_http::Http;
use robespierre_models::channels::Message;
use robespierre_models::channels::MessageContent;
# use robespierre_models::events::ReadyEvent;

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
# 
#     let connection = Connection::connect(&auth).await?;
# 
#     let context = Context::new(http, robespierre::typemap::ShareMap::custom()).with_cache(CacheConfig::default());
# 
#     let handler = Handler;
# 
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

Now, let's run it:

```bash
TOKEN=... cargo run
```

For the source code of this, see the [ping-reply-pong example](https://github.com/dblanovschi/robespierre/tree/main/examples/ping-reply-pong).
