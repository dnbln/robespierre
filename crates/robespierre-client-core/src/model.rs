#[cfg(feature = "cache")]
use robespierre_cache::CommitToCache;
use robespierre_http::HasHttp;
use robespierre_models::{
    autumn::AttachmentId,
    channels::{Channel, Message, ReplyData},
    id::{ChannelId, MemberId, ServerId, UserId},
    servers::{Member, Server},
    users::User,
};

use crate::{CacheHttp, Result};

use self::user_opt_member::UserOptMember;

pub mod mention;
pub mod user_opt_member;

pub trait IntoString: Into<String> + Send + Sync {}
impl<T> IntoString for T where T: Into<String> + Send + Sync {}

// commit_to_cache implementation when there is no cache
#[cfg(not(feature = "cache"))]
trait CommitToCache {
    fn commit_to_cache<T>(self, cache: T) -> std::future::Ready<Self>
    where
        Self: Sized,
    {
        std::future::ready(self)
    }
}

#[cfg(not(feature = "cache"))]
impl<T> CommitToCache for T {}

#[async_trait::async_trait]
pub trait MessageExt {
    async fn reply(
        &self,
        ctx: &impl HasHttp,
        content: impl IntoString + 'async_trait,
    ) -> Result<Message>;
    async fn reply_ping(
        &self,
        ctx: &impl HasHttp,
        content: impl IntoString + 'async_trait,
    ) -> Result<Message>;
    async fn author(&self, ctx: &impl CacheHttp) -> Result<User>;
    async fn member(&self, ctx: &impl CacheHttp) -> Result<Option<Member>>;
    async fn channel(&self, ctx: &impl CacheHttp) -> Result<Channel>;
    async fn server_id(&self, ctx: &impl CacheHttp) -> Result<Option<ServerId>>;
    async fn server(&self, ctx: &impl CacheHttp) -> Result<Option<Server>>;
    async fn author_user_opt_member(&self, ctx: &impl CacheHttp) -> Result<UserOptMember>;
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
    async fn reply(
        &self,
        ctx: &impl HasHttp,
        content: impl IntoString + 'async_trait,
    ) -> Result<Message> {
        let content = content.into();
        self.channel
            .send_message(ctx, |m| {
                m.content(content).reply(ReplyData {
                    id: self.id,
                    mention: false,
                })
            })
            .await
    }

    async fn reply_ping(
        &self,
        ctx: &impl HasHttp,
        content: impl IntoString + 'async_trait,
    ) -> Result<Message> {
        let content = content.into();
        self.channel
            .send_message(ctx, |m| {
                m.content(content).reply(ReplyData {
                    id: self.id,
                    mention: false,
                })
            })
            .await
    }

    async fn author(&self, ctx: &impl CacheHttp) -> Result<User> {
        self.author.user(ctx).await
    }

    async fn member(&self, ctx: &impl CacheHttp) -> Result<Option<Member>> {
        let server = self.server_id(ctx).await?;

        match server {
            Some(id) => Ok(Some(id.member(ctx, self.author).await?)),
            None => Ok(None),
        }
    }

    async fn channel(&self, ctx: &impl CacheHttp) -> Result<Channel> {
        self.channel.channel(ctx).await
    }

    async fn server_id(&self, ctx: &impl CacheHttp) -> Result<Option<ServerId>> {
        self.channel.server_id(ctx).await
    }

    async fn server(&self, ctx: &impl CacheHttp) -> Result<Option<Server>> {
        let ch = self.channel(ctx).await?;

        Ok(ch.server(ctx).await?)
    }

    async fn author_user_opt_member(&self, ctx: &impl CacheHttp) -> Result<UserOptMember> {
        let user = self.author(ctx).await?;

        let member = self.member(ctx).await.ok().flatten();

        Ok(UserOptMember { user, member })
    }
}

#[async_trait::async_trait]
pub trait ChannelExt {
    async fn server(&self, ctx: &impl CacheHttp) -> Result<Option<Server>>;
}

#[async_trait::async_trait]
impl ChannelExt for Channel {
    async fn server(&self, ctx: &impl CacheHttp) -> Result<Option<Server>> {
        let server_id = match self.server_id() {
            Some(id) => id,
            None => return Ok(None),
        };

        Ok(Some(server_id.server(ctx).await?))
    }
}

#[async_trait::async_trait]
pub trait ChannelIdExt {
    async fn channel(&self, ctx: &impl CacheHttp) -> Result<Channel>;
    async fn server_id(&self, ctx: &impl CacheHttp) -> Result<Option<ServerId>>;
    async fn server(&self, ctx: &impl CacheHttp) -> Result<Option<Server>>;

    async fn send_message<F>(&self, ctx: &impl HasHttp, message: F) -> Result<Message>
    where
        F: for<'a> FnOnce(&'a mut CreateMessage) -> &'a CreateMessage + Send;

    #[deprecated(note = "Use robespierre::model_ext::ChannelIdExt2::start_typing instead")]
    fn start_typing(&self) {}
}

#[async_trait::async_trait]
impl ChannelIdExt for ChannelId {
    async fn channel(&self, ctx: &impl CacheHttp) -> Result<Channel> {
        #[cfg(feature = "cache")]
        if let Some(cache) = ctx.cache() {
            if let Some(channel) = cache.get_channel(*self).await {
                return Ok(channel);
            }
        }

        Ok(ctx
            .http()
            .fetch_channel(*self)
            .await?
            .commit_to_cache(ctx)
            .await)
    }

    async fn server_id(&self, ctx: &impl CacheHttp) -> Result<Option<ServerId>> {
        Ok(self.channel(ctx).await?.server_id())
    }

    async fn server(&self, ctx: &impl CacheHttp) -> Result<Option<Server>> {
        self.channel(ctx).await?.server(ctx).await
    }

    async fn send_message<F>(&self, http: &impl HasHttp, message: F) -> Result<Message>
    where
        F: for<'a> FnOnce(&'a mut CreateMessage) -> &'a CreateMessage + Send,
    {
        let mut m = CreateMessage::default();
        message(&mut m);

        Ok(http
            .get_http()
            .send_message(
                *self,
                m.content,
                rusty_ulid::generate_ulid_string(),
                m.attachments,
                m.replies,
            )
            .await?)
    }
}

#[async_trait::async_trait]
pub trait ServerIdExt {
    async fn server(&self, ctx: &impl CacheHttp) -> Result<Server>;
    async fn member(&self, ctx: &impl CacheHttp, user: UserId) -> Result<Member>;
}

#[async_trait::async_trait]
impl ServerIdExt for ServerId {
    async fn server(&self, ctx: &impl CacheHttp) -> Result<Server> {
        #[cfg(feature = "cache")]
        if let Some(cache) = ctx.cache() {
            if let Some(server) = cache.get_server(*self).await {
                return Ok(server);
            }
        }

        Ok(ctx
            .http()
            .fetch_server(*self)
            .await?
            .commit_to_cache(ctx)
            .await)
    }

    async fn member(&self, ctx: &impl CacheHttp, user: UserId) -> Result<Member> {
        MemberId {
            user,
            server: *self,
        }
        .member(ctx)
        .await
    }
}

#[async_trait::async_trait]
pub trait UserIdExt {
    async fn user(&self, ctx: &impl CacheHttp) -> Result<User>;
}

#[async_trait::async_trait]
impl UserIdExt for UserId {
    async fn user(&self, ctx: &impl CacheHttp) -> Result<User> {
        #[cfg(feature = "cache")]
        if let Some(cache) = ctx.cache() {
            if let Some(user) = cache.get_user(*self).await {
                return Ok(user);
            }
        }

        Ok(ctx
            .http()
            .fetch_user(*self)
            .await?
            .commit_to_cache(ctx)
            .await)
    }
}

#[async_trait::async_trait]
pub trait MemberIdExt {
    async fn member(&self, ctx: &impl CacheHttp) -> Result<Member>;
}

#[async_trait::async_trait]
impl MemberIdExt for MemberId {
    async fn member(&self, ctx: &impl CacheHttp) -> Result<Member> {
        #[cfg(feature = "cache")]
        if let Some(cache) = ctx.cache() {
            if let Some(member) = cache.get_member(*self).await {
                return Ok(member);
            }
        }

        Ok(ctx
            .http()
            .fetch_member(self.server, self.user)
            .await?
            .commit_to_cache(ctx)
            .await)
    }
}
