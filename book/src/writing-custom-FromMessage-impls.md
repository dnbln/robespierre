# Writing custom `FromMessage` impls

Let's start with the basic imports:

```rust ,no_run
use std::future::{Ready, ready};

use robespierre::framework::standard::{FwContext, CommandResult, CommandError};
use robespierre::framework::standard::extractors::{Msg, FromMessage};

# #[tokio::main]
# async fn main() {}
```

Let's say we want to get the whole message content:

```rust ,no_run
# use std::future::{Ready, ready};
# 
# use robespierre::framework::standard::{FwContext, CommandResult, CommandError};
# use robespierre::framework::standard::extractors::{Msg, FromMessage};
use robespierre_models::channel::MessageContent;

# #[tokio::main]
# async fn main() {}

pub struct WholeMessageContent(pub String);

#[derive(Debug, thiserror::Error)]
#[error("message has no content")]
struct HasNoContent;

impl FromMessage for WholeMessageContent {
    type Config = ();
    type Fut = Ready<CommandResult<Self>>;

    fn from_message(ctx: FwContext, msg: Msg, _config: Self::Config) -> Self::Fut {
        let result = match &msg.message.content {
            MessageContent::Content(s) => Ok::<_, CommandError>(Self(s.clone())),
            _ => Err(HasNoContent.into()),
        };

        ready(result)
    }
}
```

And then everything you have to do is use it:

```rust ,no_run
# use std::future::{Ready, ready};
# 
# use robespierre::framework::standard::{FwContext, CommandResult, CommandError};
# use robespierre::framework::standard::extractors::{Msg, FromMessage};
use robespierre::framework::standard::macros::{command};
use robespierre::model::MessageExt;
# use robespierre_models::channel::MessageContent;
use robespierre_models::channel::Message;

# #[tokio::main]
# async fn main() {}
# 
# pub struct WholeMessageContent(pub String);
# 
# #[derive(Debug, thiserror::Error)]
# #[error("message has no content")]
# struct HasNoContent;
# 
# impl FromMessage for WholeMessageContent {
#     type Config = ();
#     type Fut = Ready<CommandResult<Self>>;
# 
#     fn from_message(ctx: FwContext, msg: Msg, _config: Self::Config) -> Self::Fut {
#         let result = match &msg.message.content {
#             MessageContent::Content(s) => Ok::<_, CommandError>(Self(s.clone())),
#             _ => Err(HasNoContent.into()),
#         };
# 
#         ready(result)
#     }
# }

#[command]
async fn cmd(ctx: &FwContext, message: &Message, WholeMessageContent(content): WholeMessageContent) -> CommandResult {
    message.reply(ctx, format!(r#"Whole message content is "{}""#, content));
    Ok(())
}
```