pub extern crate async_std;
pub extern crate typemap;

use std::sync::Arc;

#[cfg(feature = "framework")]
use framework::Framework;
#[cfg(feature = "cache")]
use robespierre_cache::{Cache, CacheConfig, CommitToCache, HasCache};
#[cfg(feature = "events")]
use robespierre_events::{
    typing::TypingSession, ConnectionMessage, ConnectionMessanger, EventsError, RawEventHandler,
};
use robespierre_http::{Http, HttpAuthentication, HttpError};
use robespierre_models::{
    channel::{Channel, ChannelField, Message, PartialChannel, PartialMessage},
    events::{ReadyEvent, ServerToClientEvent},
    id::{ChannelId, MemberId, MessageId, RoleId, ServerId, UserId},
    server::{MemberField, PartialMember, PartialRole, PartialServer, RoleField, ServerField},
    user::{PartialUser, RelationshipStatus, UserField},
};

pub use async_trait::async_trait;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use typemap::ShareMap;

#[cfg(feature = "framework")]
pub mod framework;
pub mod model;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http error")]
    Http(#[from] HttpError),
    #[cfg(feature = "events")]
    #[error("events error")]
    Events(#[from] EventsError),
}

pub type Result<T = ()> = std::result::Result<T, Error>;

/// A high-level event handler. Defines handlers for all the different types of events.
#[cfg(feature = "events")]
#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait EventHandler: Send + Sync {
    /// Gets called when the [`ReadyEvent`] is received.
    async fn on_ready(&self, ctx: Context, ready: ReadyEvent) {}
    /// Gets called when a message is received.
    async fn on_message(&self, ctx: Context, message: Message) {}
    /// Gets called when a message is updated.
    async fn on_message_update(
        &self,
        ctx: Context,
        channel: ChannelId,
        message: MessageId,
        modifications: PartialMessage,
    ) {
    }
    /// Gets called when a message is deleted.
    async fn on_message_delete(&self, ctx: Context, channel_id: ChannelId, message_id: MessageId) {}
    /// Gets called when a channel is created.
    async fn on_channel_create(&self, ctx: Context, channel: Channel) {}
    /// Gets called when a channel is updated.
    async fn on_channel_update(
        &self,
        ctx: Context,
        channel_id: ChannelId,
        modifications: PartialChannel,
        remove: Option<ChannelField>,
    ) {
    }
    /// Gets called when a channel is deleted.
    async fn on_channel_delete(&self, ctx: Context, channel_id: ChannelId) {}
    /// Gets called when an user joins a group.
    async fn on_group_join(&self, ctx: Context, id: ChannelId, user: UserId) {}
    /// Gets called when an user leaves a group.
    async fn on_group_leave(&self, ctx: Context, id: ChannelId, user: UserId) {}
    /// Gets called when someone starts typing in a channel.
    async fn on_start_typing(&self, ctx: Context, channel: ChannelId, user: UserId) {}
    /// Gets called when someone stops typing in a channel.
    async fn on_stop_typing(&self, ctx: Context, channel: ChannelId, user: UserId) {}
    /// Gets called when a server is updated.
    async fn on_server_update(
        &self,
        ctx: Context,
        server: ServerId,
        modifications: PartialServer,
        remove: Option<ServerField>,
    ) {
    }
    /// Gets called when a server is deleted.
    ///
    /// Could mean the user / bot was kicked, banned, or otherwise left it, not necessarily
    /// that it was deleted by the owner and doesn't exist anymore.
    async fn on_server_delete(&self, ctx: Context, server: ServerId) {}
    /// Gets called when an user joins a server.
    async fn on_server_member_join(&self, ctx: Context, server: ServerId, user: UserId) {}
    /// Gets called when a member is updated inside a server.
    async fn on_server_member_update(
        &self,
        ctx: Context,
        member: MemberId,
        modifications: PartialMember,
        remove: Option<MemberField>,
    ) {
    }
    /// Gets called when a member leaves a server.
    async fn on_server_member_leave(&self, ctx: Context, server: ServerId, user: UserId) {}
    /// Gets called when a server role is updated.
    async fn on_server_role_update(
        &self,
        ctx: Context,
        server: ServerId,
        role: RoleId,
        modifications: PartialRole,
        remove: Option<RoleField>,
    ) {
    }
    /// Gets called when a server role is deleted.
    async fn on_server_role_delete(&self, ctx: Context, server: ServerId, role: RoleId) {}
    /// Gets called when an user is updated.
    async fn on_user_update(
        &self,
        ctx: Context,
        id: UserId,
        modifications: PartialUser,
        remove: Option<UserField>,
    ) {
    }
    /// Gets called when the relationship with an user is updated.
    async fn on_user_relationship_update(
        &self,
        ctx: Context,
        self_id: UserId,
        other_id: UserId,
        status: RelationshipStatus,
    ) {
    }
}

/// Wraps an event handler, updating the cache and then forwarding the events
/// to the wrapped [`EventHandler`]
#[cfg(all(feature = "events", feature = "cache"))]
#[derive(Clone)]
pub struct CacheWrap<Ctx: HasCache + Clone + 'static, T: RawEventHandler<Context = Ctx>>(T);

#[cfg(all(feature = "events", feature = "cache"))]
impl<Ctx, T> CacheWrap<Ctx, T>
where
    Ctx: HasCache + Clone + 'static,
    T: RawEventHandler<Context = Ctx>,
{
    /// Creates a new [`CacheWrap`]
    pub fn new(wrapped: T) -> Self {
        Self(wrapped)
    }
}

#[cfg(all(feature = "events", feature = "cache"))]
#[async_trait::async_trait]
impl<T, Ctx: HasCache + Clone + 'static> RawEventHandler for CacheWrap<Ctx, T>
where
    T: RawEventHandler<Context = Ctx>,
{
    type Context = Ctx;

    async fn handle(self, ctx: Self::Context, event: ServerToClientEvent) {
        event.commit_to_cache_ref(&ctx).await;

        self.0.handle(ctx, event).await
    }
}

#[cfg(all(feature = "events", feature = "framework"))]
#[derive(Clone)]
pub struct FrameworkWrap<
    FrameworkContext: From<Context> + Send + Sync + 'static,
    T: EventHandler + Clone + 'static,
> {
    fw: Arc<RwLock<Box<dyn Framework<Context = FrameworkContext> + 'static>>>,
    handler: T,
}

#[cfg(all(feature = "events", feature = "framework"))]
impl<
        FrameworkContext: From<Context> + Send + Sync + 'static,
        T: EventHandler + Clone + 'static,
    > FrameworkWrap<FrameworkContext, T>
{
    pub fn new<Fw: Framework<Context = FrameworkContext> + 'static>(fw: Fw, handler: T) -> Self {
        Self {
            fw: Arc::new(RwLock::new(Box::new(fw))),
            handler,
        }
    }
}

#[cfg(all(feature = "events", feature = "framework"))]
#[async_trait::async_trait]
impl<
        FrameworkContext: From<Context> + Send + Sync + 'static,
        T: EventHandler + Clone + 'static,
    > EventHandler for FrameworkWrap<FrameworkContext, T>
{
    async fn on_ready(&self, ctx: Context, ready: ReadyEvent) {
        self.handler.on_ready(ctx, ready).await
    }

    async fn on_message(&self, ctx: Context, message: Message) {
        self.fw
            .read()
            .await
            .handle(FrameworkContext::from(ctx.clone()), &message)
            .await;
        self.handler.on_message(ctx, message).await
    }

    async fn on_message_update(
        &self,
        ctx: Context,
        channel: ChannelId,
        message: MessageId,
        modifications: PartialMessage,
    ) {
        self.handler
            .on_message_update(ctx, channel, message, modifications)
            .await
    }

    async fn on_message_delete(&self, ctx: Context, channel_id: ChannelId, message_id: MessageId) {
        self.handler
            .on_message_delete(ctx, channel_id, message_id)
            .await
    }

    async fn on_channel_create(&self, ctx: Context, channel: Channel) {
        self.handler.on_channel_create(ctx, channel).await
    }

    async fn on_channel_update(
        &self,
        ctx: Context,
        channel_id: ChannelId,
        modifications: PartialChannel,
        remove: Option<ChannelField>,
    ) {
        self.handler
            .on_channel_update(ctx, channel_id, modifications, remove)
            .await
    }

    async fn on_channel_delete(&self, ctx: Context, channel_id: ChannelId) {
        self.handler.on_channel_delete(ctx, channel_id).await
    }

    async fn on_group_join(&self, ctx: Context, id: ChannelId, user: UserId) {
        self.handler.on_group_join(ctx, id, user).await
    }

    async fn on_group_leave(&self, ctx: Context, id: ChannelId, user: UserId) {
        self.handler.on_group_leave(ctx, id, user).await
    }

    async fn on_start_typing(&self, ctx: Context, channel: ChannelId, user: UserId) {
        self.handler.on_start_typing(ctx, channel, user).await
    }

    async fn on_stop_typing(&self, ctx: Context, channel: ChannelId, user: UserId) {
        self.handler.on_stop_typing(ctx, channel, user).await
    }

    async fn on_server_update(
        &self,
        ctx: Context,
        server: ServerId,
        modifications: PartialServer,
        remove: Option<ServerField>,
    ) {
        self.handler
            .on_server_update(ctx, server, modifications, remove)
            .await
    }

    async fn on_server_delete(&self, ctx: Context, server: ServerId) {
        self.handler.on_server_delete(ctx, server).await
    }

    async fn on_server_member_join(&self, ctx: Context, server: ServerId, user: UserId) {
        self.handler.on_server_member_join(ctx, server, user).await
    }

    async fn on_server_member_update(
        &self,
        ctx: Context,
        member: MemberId,
        modifications: PartialMember,
        remove: Option<MemberField>,
    ) {
        self.handler
            .on_server_member_update(ctx, member, modifications, remove)
            .await
    }

    async fn on_server_member_leave(&self, ctx: Context, server: ServerId, user: UserId) {
        self.handler.on_server_member_leave(ctx, server, user).await
    }

    async fn on_server_role_update(
        &self,
        ctx: Context,
        server: ServerId,
        role: RoleId,
        modifications: PartialRole,
        remove: Option<RoleField>,
    ) {
        self.handler
            .on_server_role_update(ctx, server, role, modifications, remove)
            .await
    }

    async fn on_server_role_delete(&self, ctx: Context, server: ServerId, role: RoleId) {
        self.handler.on_server_role_delete(ctx, server, role).await
    }

    async fn on_user_update(
        &self,
        ctx: Context,
        id: UserId,
        modifications: PartialUser,
        remove: Option<UserField>,
    ) {
        self.handler
            .on_user_update(ctx, id, modifications, remove)
            .await
    }

    async fn on_user_relationship_update(
        &self,
        ctx: Context,
        self_id: UserId,
        other_id: UserId,
        status: RelationshipStatus,
    ) {
        self.handler
            .on_user_relationship_update(ctx, self_id, other_id, status)
            .await
    }
}

/// An object that can be passed to [`robespierre_events::Connection::run`], and
/// distinguishes between the events and calls the relevant handler.
#[cfg(feature = "events")]
#[derive(Clone)]
pub struct EventHandlerWrap<T: EventHandler + Clone + 'static>(T);

#[cfg(feature = "events")]
impl<T: EventHandler + Clone + 'static> EventHandlerWrap<T> {
    pub fn new(handler: T) -> Self {
        Self(handler)
    }
}

#[cfg(feature = "events")]
#[async_trait::async_trait]
impl<T> RawEventHandler for EventHandlerWrap<T>
where
    T: EventHandler + Clone + 'static,
{
    type Context = Context;

    async fn handle(self, ctx: Self::Context, event: ServerToClientEvent) {
        match event {
            ServerToClientEvent::Error { error } => tracing::error!("Error: {}", error),
            ServerToClientEvent::Authenticated => {}
            ServerToClientEvent::Pong { time } => {
                tracing::debug!("Got a pong from the server, time: {}", time)
            }
            ServerToClientEvent::Ready { event } => self.0.on_ready(ctx, event).await,
            ServerToClientEvent::Message { message } => self.0.on_message(ctx, message).await,
            ServerToClientEvent::MessageUpdate { id, channel, data } => {
                self.0.on_message_update(ctx, channel, id, data).await
            }
            ServerToClientEvent::MessageDelete { id, channel } => {
                self.0.on_message_delete(ctx, channel, id).await
            }
            ServerToClientEvent::ChannelCreate { channel } => {
                self.0.on_channel_create(ctx, channel).await
            }
            ServerToClientEvent::ChannelUpdate { id, data, clear } => {
                self.0.on_channel_update(ctx, id, data, clear).await
            }
            ServerToClientEvent::ChannelDelete { id } => self.0.on_channel_delete(ctx, id).await,
            ServerToClientEvent::ChannelGroupJoin { id, user } => {
                self.0.on_group_join(ctx, id, user).await
            }
            ServerToClientEvent::ChannelGroupLeave { id, user } => {
                self.0.on_group_leave(ctx, id, user).await
            }
            ServerToClientEvent::ChannelStartTyping { id, user } => {
                self.0.on_start_typing(ctx, id, user).await
            }
            ServerToClientEvent::ChannelStopTyping { id, user } => {
                self.0.on_stop_typing(ctx, id, user).await
            }
            ServerToClientEvent::ChannelAck {
                id,
                user,
                message_id,
            } => tracing::debug!("Got ack ch={}, user={}, message={}", id, user, message_id),
            ServerToClientEvent::ServerUpdate { id, data, clear } => {
                self.0.on_server_update(ctx, id, data, clear).await
            }
            ServerToClientEvent::ServerDelete { id } => self.0.on_server_delete(ctx, id).await,
            ServerToClientEvent::ServerMemberUpdate { id, data, clear } => {
                self.0.on_server_member_update(ctx, id, data, clear).await
            }
            ServerToClientEvent::ServerMemberJoin { id, user } => {
                self.0.on_server_member_join(ctx, id, user).await
            }
            ServerToClientEvent::ServerMemberLeave { id, user } => {
                self.0.on_server_member_leave(ctx, id, user).await
            }
            ServerToClientEvent::ServerRoleUpdate {
                id,
                role_id,
                data,
                clear,
            } => {
                self.0
                    .on_server_role_update(ctx, id, role_id, data, clear)
                    .await
            }
            ServerToClientEvent::ServerRoleDelete { id, role_id } => {
                self.0.on_server_role_delete(ctx, id, role_id).await
            }
            ServerToClientEvent::UserUpdate { id, data, clear } => {
                self.0.on_user_update(ctx, id, data, clear).await
            }
            ServerToClientEvent::UserRelationship { id, user, status } => {
                self.0
                    .on_user_relationship_update(ctx, id, user, status)
                    .await
            }
        }
    }
}

pub enum Authentication {
    Bot {
        token: String,
    },
    User {
        user_id: UserId,
        session_token: String,
    },
}

impl Authentication {
    pub fn bot(token: impl Into<String>) -> Self {
        Self::Bot {
            token: token.into(),
        }
    }

    pub fn user(user_id: UserId, session_token: impl Into<String>) -> Self {
        Self::User {
            user_id,
            session_token: session_token.into(),
        }
    }
}

#[cfg(feature = "events")]
impl<'a> From<&'a Authentication> for robespierre_events::Authentication<'a> {
    fn from(auth: &'a Authentication) -> Self {
        match auth {
            Authentication::Bot { token } => Self::Bot {
                token: token.as_str(),
            },
            Authentication::User {
                user_id,
                session_token,
            } => Self::User {
                user_id: *user_id,
                session_token: session_token.as_str(),
            },
        }
    }
}

impl<'a> From<&'a Authentication> for HttpAuthentication<'a> {
    fn from(auth: &'a Authentication) -> Self {
        match auth {
            Authentication::Bot { token } => Self::BotToken {
                token: token.as_str(),
            },
            Authentication::User {
                user_id,
                session_token,
            } => Self::UserSession {
                user_id: *user_id,
                session_token: session_token.as_str(),
            },
        }
    }
}

#[derive(Clone)]
pub struct Context {
    pub http: Arc<Http>,
    #[cfg(feature = "cache")]
    pub cache: Option<Arc<Cache>>,
    pub data: Arc<RwLock<ShareMap>>,
    #[cfg(feature = "events")]
    messanger: Option<ConnectionMessanger>,
}

impl AsRef<Context> for Context {
    fn as_ref(&self) -> &Context {
        self
    }
}

#[async_trait::async_trait]
pub trait UserData {
    async fn data_lock_read(&self) -> RwLockReadGuard<ShareMap>;
    async fn data_lock_write(&self) -> RwLockWriteGuard<ShareMap>;
}

#[async_trait::async_trait]
impl UserData for Context {
    async fn data_lock_read(&self) -> RwLockReadGuard<ShareMap> {
        self.data.read().await
    }

    async fn data_lock_write(&self) -> RwLockWriteGuard<ShareMap> {
        self.data.write().await
    }
}

impl Context {
    pub fn new(http: Http, typemap: impl Into<ShareMap>) -> Self {
        Self {
            http: Arc::new(http),
            #[cfg(feature = "cache")]
            cache: None,
            data: Arc::new(RwLock::new(typemap.into())),
            #[cfg(feature = "events")]
            messanger: None,
        }
    }

    #[cfg(feature = "cache")]
    pub fn with_cache(self, cache_config: CacheConfig) -> Self {
        Self {
            cache: Some(Cache::new(cache_config)),
            ..self
        }
    }

    #[cfg(feature = "events")]
    pub(crate) fn start_typing(&self, channel: ChannelId) -> TypingSession {
        let messanger = self.messanger.as_ref().expect(
            "need messager; did you forget to call .set_messager(...) on robespierre::Context?",
        );

        messanger.send(ConnectionMessage::StartTyping { channel });

        TypingSession::new(channel, messanger.clone())
    }
}

#[cfg(feature = "events")]
impl robespierre_events::Context for Context {
    fn set_messanger(self, messanger: ConnectionMessanger) -> Self {
        Self {
            messanger: Some(messanger),
            ..self
        }
    }
}

#[cfg(feature = "cache")]
impl HasCache for Context {
    fn get_cache(&self) -> Option<&Cache> {
        self.cache.as_ref().map(|it| &**it)
    }
}

pub trait HasHttp: Send + Sync {
    fn get_http(&self) -> &Http;
}

impl HasHttp for Context {
    fn get_http(&self) -> &Http {
        &*self.http
    }
}

impl HasHttp for Http {
    fn get_http(&self) -> &Http {
        self
    }
}

#[cfg(feature = "cache")]
pub trait CacheHttp: HasCache {
    fn http(&self) -> &Http;
    fn cache(&self) -> Option<&Cache>;
}

#[cfg(feature = "cache")]
impl<T: HasCache + HasHttp> CacheHttp for T {
    fn http(&self) -> &Http {
        self.get_http()
    }

    fn cache(&self) -> Option<&Cache> {
        self.get_cache()
    }
}

#[cfg(not(feature = "cache"))]
pub trait CacheHttp {
    fn http(&self) -> &Http;
}

#[cfg(not(feature = "cache"))]
impl<T: HasHttp> CacheHttp for T {
    fn http(&self) -> &Http {
        self.get_http()
    }
}
