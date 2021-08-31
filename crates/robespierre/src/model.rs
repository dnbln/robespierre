use robespierre_cache::CommitToCache;
use robespierre_models::{
    channel::{Channel, Message, ReplyData},
    id::{AttachmentId, ChannelId, ServerId, UserId},
    server::Server,
    user::User,
};

use crate::{Context, Result};

pub mod mention;

pub trait IntoString: Into<String> + Send + Sync + 'static {}
impl<T> IntoString for T where T: Into<String> + Send + Sync + 'static {}

#[async_trait::async_trait]
pub trait MessageExt {
    async fn reply(&self, ctx: &Context, content: impl IntoString) -> Result<Message>;
    async fn reply_ping(&self, ctx: &Context, content: impl IntoString) -> Result<Message>;
    async fn author(&self, ctx: &Context) -> Result<User>;
    async fn channel(&self, ctx: &Context) -> Result<Channel>;
    async fn server_id(&self, ctx: &Context) -> Result<Option<ServerId>>;
    async fn server(&self, ctx: &Context) -> Result<Option<Server>>;
}

#[derive(Debug, Clone, Default)]
pub struct CreateMessage {
    content: String,
    attachments: Vec<AttachmentId>,
    replies: Vec<ReplyData>,
}

impl CreateMessage {
    pub fn content(&mut self, content: impl Into<String>) -> &mut Self {
        self.content = content.into();
        self
    }

    pub fn attachments(&mut self, attachments: Vec<AttachmentId>) -> &mut Self {
        self.attachments.extend(attachments.into_iter());
        self
    }

    pub fn attachment(&mut self, attachment: AttachmentId) -> &mut Self {
        self.attachments.push(attachment);
        self
    }

    pub fn replies(&mut self, replies: Vec<ReplyData>) -> &mut Self {
        self.replies.extend(replies.into_iter());
        self
    }

    pub fn reply(&mut self, reply: impl Into<ReplyData>) -> &mut Self {
        self.replies.push(reply.into());
        self
    }
}

#[async_trait::async_trait]
impl MessageExt for Message {
    async fn reply(&self, ctx: &Context, content: impl IntoString) -> Result<Message> {
        self.channel
            .send_message(ctx, |m| {
                m.content(content).reply(ReplyData {
                    id: self.id,
                    mention: false,
                })
            })
            .await
    }

    async fn reply_ping(&self, ctx: &Context, content: impl IntoString) -> Result<Message> {
        self.channel
            .send_message(ctx, |m| {
                m.content(content).reply(ReplyData {
                    id: self.id,
                    mention: false,
                })
            })
            .await
    }

    async fn author(&self, ctx: &Context) -> Result<User> {
        self.author.user(ctx).await
    }

    async fn channel(&self, ctx: &Context) -> Result<Channel> {
        self.channel.channel(ctx).await
    }

    async fn server_id(&self, ctx: &Context) -> Result<Option<ServerId>> {
        self.channel.server_id(ctx).await
    }

    async fn server(&self, ctx: &Context) -> Result<Option<Server>> {
        let ch = self.channel(ctx).await?;

        Ok(ch.server(ctx).await?)
    }
}

#[async_trait::async_trait]
pub trait ChannelExt {
    async fn server(&self, ctx: &Context) -> Result<Option<Server>>;
}

#[async_trait::async_trait]
impl ChannelExt for Channel {
    async fn server(&self, ctx: &Context) -> Result<Option<Server>> {
        let server_id = match self.server_id() {
            Some(id) => id,
            None => return Ok(None),
        };

        Ok(Some(server_id.server(ctx).await?))
    }
}

#[async_trait::async_trait]
pub trait ChannelIdExt {
    async fn channel(&self, ctx: &Context) -> Result<Channel>;
    async fn server_id(&self, ctx: &Context) -> Result<Option<ServerId>>;
    async fn server(&self, ctx: &Context) -> Result<Option<Server>>;

    async fn send_message<F>(&self, ctx: &Context, message: F) -> Result<Message>
    where
        F: for<'a> FnOnce(&'a mut CreateMessage) -> &'a CreateMessage + Send;

    /// timeout in ~3 seconds, call this function again if typing for a longer period of time
    fn start_typing(&self, ctx: &Context);
    fn stop_typing(&self, ctx: &Context);
}

#[async_trait::async_trait]
impl ChannelIdExt for ChannelId {
    async fn channel(&self, ctx: &Context) -> Result<Channel> {
        if let Some(ref cache) = ctx.cache {
            if let Some(channel) = cache.get_channel(*self).await {
                return Ok(channel);
            }
        }

        Ok(ctx
            .http
            .fetch_channel(*self)
            .await?
            .commit_to_cache(ctx)
            .await)
    }

    async fn server_id(&self, ctx: &Context) -> Result<Option<ServerId>> {
        Ok(self.channel(ctx).await?.server_id())
    }

    async fn server(&self, ctx: &Context) -> Result<Option<Server>> {
        self.channel(ctx).await?.server(ctx).await
    }

    async fn send_message<F>(&self, ctx: &Context, message: F) -> Result<Message>
    where
        F: for<'a> FnOnce(&'a mut CreateMessage) -> &'a CreateMessage + Send,
    {
        let mut m = CreateMessage::default();
        message(&mut m);

        Ok(ctx
            .http
            .send_message(
                *self,
                m.content,
                rusty_ulid::generate_ulid_string(),
                m.attachments,
                m.replies,
            )
            .await?)
    }

    fn start_typing(&self, ctx: &Context) {
        ctx.start_typing(*self)
    }

    fn stop_typing(&self, ctx: &Context) {
        ctx.stop_typing(*self)
    }
}

#[async_trait::async_trait]
pub trait ServerIdExt {
    async fn server(&self, ctx: &Context) -> Result<Server>;
}

#[async_trait::async_trait]
impl ServerIdExt for ServerId {
    async fn server(&self, ctx: &Context) -> Result<Server> {
        if let Some(ref cache) = ctx.cache {
            if let Some(server) = cache.get_server(*self).await {
                return Ok(server);
            }
        }

        Ok(ctx
            .http
            .fetch_server(*self)
            .await?
            .commit_to_cache(ctx)
            .await)
    }
}

#[async_trait::async_trait]
pub trait UserIdExt {
    async fn user(&self, ctx: &Context) -> Result<User>;
}

#[async_trait::async_trait]
impl UserIdExt for UserId {
    async fn user(&self, ctx: &Context) -> Result<User> {
        if let Some(ref cache) = ctx.cache {
            if let Some(user) = cache.get_user(*self).await {
                return Ok(user);
            }
        }

        Ok(ctx.http.fetch_user(*self).await?.commit_to_cache(ctx).await)
    }
}
