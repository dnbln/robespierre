use robespierre_cache::CommitToCache;
use robespierre_models::{channel::{Channel, Message, ReplyData}, user::User};

use crate::{Context, Result};

pub trait AsRefStr: AsRef<str> + Send + Sync + 'static {}
impl<T> AsRefStr for T where T: AsRef<str> + Send + Sync + 'static {}

#[async_trait::async_trait]
pub trait MessageExt {
    async fn reply(&self, ctx: &Context, content: impl AsRefStr) -> Result<Message>;
    async fn reply_ping(&self, ctx: &Context, content: impl AsRefStr) -> Result<Message>;
    async fn author(&self, ctx: &Context) -> Result<User>;
    async fn channel(&self, ctx: &Context) -> Result<Channel>;
}

#[async_trait::async_trait]
impl MessageExt for Message {
    async fn reply(&self, ctx: &Context, content: impl AsRefStr) -> Result<Message> {
        Ok(ctx
            .http
            .send_message(
                self.channel,
                content.as_ref(),
                rusty_ulid::generate_ulid_string(),
                vec![],
                vec![ReplyData {
                    id: self.id,
                    mention: false,
                }],
            )
            .await?)
    }

    async fn reply_ping(&self, ctx: &Context, content: impl AsRefStr) -> Result<Message> {
        Ok(ctx
            .http
            .send_message(
                self.channel,
                content.as_ref(),
                rusty_ulid::generate_ulid_string(),
                vec![],
                vec![ReplyData {
                    id: self.id,
                    mention: true,
                }],
            )
            .await?)
    }

    async fn author(&self, ctx: &Context) -> Result<User> {
        if let Some(ref cache) = ctx.cache {
            if let Some(user) = cache.get_user(self.author).await {
                return Ok(user)
            }
        }

        Ok(ctx.http.fetch_user(self.author).await?.commit_to_cache(ctx).await)
    }

    async fn channel(&self, ctx: &Context) -> Result<Channel> {
        if let Some(ref cache) = ctx.cache {
            if let Some(channel) = cache.get_channel(self.channel).await {
                return Ok(channel);
            }
        }

        Ok(ctx.http.fetch_channel(self.channel).await?.commit_to_cache(ctx).await)
    }    
}
