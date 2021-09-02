use std::sync::Arc;

use robespierre_cache::{Cache, CacheConfig, CommitToCache, HasCache};
use robespierre_events::{ConnectionMessage, ConnectionMessanger, EventsError, RawEventHandler, ReadyEvent, ServerToClientEvent, typing::TypingSession};
use robespierre_http::{Http, HttpAuthentication, HttpError};
use robespierre_models::{
    channel::{Channel, ChannelField, Message, PartialChannel, PartialMessage},
    id::{ChannelId, MemberId, MessageId, RoleId, ServerId, UserId},
    server::{MemberField, PartialMember, PartialRole, PartialServer, RoleField, ServerField},
    user::{PartialUser, RelationshipStatus, UserField},
};

pub use async_trait::async_trait;

pub mod model;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http error")]
    Http(#[from] HttpError),
    #[error("events error")]
    Events(#[from] EventsError),
}

pub type Result<T = ()> = std::result::Result<T, Error>;


/// A high-level event handler. Defines handlers for all the different types of events.
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
#[derive(Clone)]
pub struct CacheWrap<T: EventHandler + Clone + 'static>(T);

impl<T> CacheWrap<T>
where
    T: EventHandler + Clone + 'static,
{
    /// Creates a new [`CacheWrap`]
    pub fn new(wrapped: T) -> Self {
        Self(wrapped)
    }
}

#[async_trait::async_trait]
impl<T> EventHandler for CacheWrap<T>
where
    T: EventHandler + Clone + 'static,
{
    async fn on_ready(&self, ctx: Context, ready: ReadyEvent) {
        if let Some(ref cache) = ctx.cache {
            for user in ready.users.iter() {
                user.commit_to_cache_ref(cache).await;
            }
            for server in ready.servers.iter() {
                server.commit_to_cache_ref(cache).await;
            }
            for channel in ready.channels.iter() {
                channel.commit_to_cache_ref(cache).await;
            }
            for member in ready.members.iter() {
                member.commit_to_cache_ref(cache).await;
            }
        }

        self.0.on_ready(ctx, ready).await
    }

    async fn on_message(&self, ctx: Context, message: Message) {
        if let Some(ref cache) = ctx.cache {
            message.commit_to_cache_ref(cache).await;
        }

        self.0.on_message(ctx, message).await
    }

    async fn on_message_update(
        &self,
        ctx: Context,
        channel: ChannelId,
        message: MessageId,
        modifications: PartialMessage,
    ) {
        if let Some(ref cache) = ctx.cache {
            cache
                .patch_message(channel, message, || modifications.clone())
                .await;
        }

        self.0
            .on_message_update(ctx, channel, message, modifications)
            .await
    }

    async fn on_message_delete(&self, ctx: Context, channel_id: ChannelId, message_id: MessageId) {
        self.0.on_message_delete(ctx, channel_id, message_id).await
    }

    async fn on_channel_create(&self, ctx: Context, channel: Channel) {
        if let Some(ref cache) = ctx.cache {
            channel.commit_to_cache_ref(cache).await;
        }

        self.0.on_channel_create(ctx, channel).await
    }

    async fn on_channel_update(
        &self,
        ctx: Context,
        channel_id: ChannelId,
        modifications: PartialChannel,
        remove: Option<ChannelField>,
    ) {
        if let Some(ref cache) = ctx.cache {
            cache
                .patch_channel(channel_id, || modifications.clone(), remove)
                .await;
        }

        self.0
            .on_channel_update(ctx, channel_id, modifications, remove)
            .await
    }

    async fn on_channel_delete(&self, ctx: Context, channel_id: ChannelId) {
        self.0.on_channel_delete(ctx, channel_id).await
    }

    async fn on_group_join(&self, ctx: Context, id: ChannelId, user: UserId) {
        self.0.on_group_join(ctx, id, user).await
    }

    async fn on_group_leave(&self, ctx: Context, id: ChannelId, user: UserId) {
        self.0.on_group_leave(ctx, id, user).await
    }

    async fn on_start_typing(&self, ctx: Context, channel: ChannelId, user: UserId) {
        self.0.on_start_typing(ctx, channel, user).await
    }

    async fn on_stop_typing(&self, ctx: Context, channel: ChannelId, user: UserId) {
        self.0.on_stop_typing(ctx, channel, user).await
    }

    async fn on_server_update(
        &self,
        ctx: Context,
        server: ServerId,
        modifications: PartialServer,
        remove: Option<ServerField>,
    ) {
        if let Some(ref cache) = ctx.cache {
            cache
                .patch_server(server, || modifications.clone(), remove)
                .await;
        }

        self.0
            .on_server_update(ctx, server, modifications, remove)
            .await
    }

    async fn on_server_delete(&self, ctx: Context, server: ServerId) {
        self.0.on_server_delete(ctx, server).await
    }

    async fn on_server_member_join(&self, ctx: Context, server: ServerId, user: UserId) {
        self.0.on_server_member_join(ctx, server, user).await
    }

    async fn on_server_member_update(
        &self,
        ctx: Context,
        member: MemberId,
        modifications: PartialMember,
        remove: Option<MemberField>,
    ) {
        if let Some(ref cache) = ctx.cache {
            cache
                .patch_member(member, || modifications.clone(), remove)
                .await
        }
        self.0
            .on_server_member_update(ctx, member, modifications, remove)
            .await
    }

    async fn on_server_member_leave(&self, ctx: Context, server: ServerId, user: UserId) {
        self.0.on_server_member_leave(ctx, server, user).await
    }

    async fn on_server_role_update(
        &self,
        ctx: Context,
        server: ServerId,
        role: RoleId,
        modifications: PartialRole,
        remove: Option<RoleField>,
    ) {
        self.0
            .on_server_role_update(ctx, server, role, modifications, remove)
            .await
    }

    async fn on_server_role_delete(&self, ctx: Context, server: ServerId, role: RoleId) {
        self.0.on_server_role_delete(ctx, server, role).await
    }

    async fn on_user_update(
        &self,
        ctx: Context,
        id: UserId,
        modifications: PartialUser,
        remove: Option<UserField>,
    ) {
        if let Some(ref cache) = ctx.cache {
            cache.patch_user(id, || modifications.clone(), remove).await;
        }
        self.0.on_user_update(ctx, id, modifications, remove).await
    }

    async fn on_user_relationship_update(
        &self,
        ctx: Context,
        self_id: UserId,
        other_id: UserId,
        status: RelationshipStatus,
    ) {
        self.0
            .on_user_relationship_update(ctx, self_id, other_id, status)
            .await
    }
}

/// An object that can be passed to [`robespierre_events::Connection::run`], and
/// distinguishes between the events and calls the relevant handler.
#[derive(Clone)]
pub struct EventHandlerWrap<T: EventHandler + Clone + 'static>(T);

impl<T: EventHandler + Clone + 'static> EventHandlerWrap<T> {
    pub fn new(handler: T) -> Self {
        Self(handler)
    }
}

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

#[derive(Clone)]
pub struct Context {
    pub http: Arc<Http>,
    pub cache: Option<Arc<Cache>>,
    messanger: Option<ConnectionMessanger>,
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

impl Context {
    pub fn new(http: Http) -> Self {
        Self {
            http: Arc::new(http),
            cache: None,
            messanger: None,
        }
    }

    pub fn with_cache(self, cache_config: CacheConfig) -> Self {
        Self {
            cache: Some(Cache::new(cache_config)),
            ..self
        }
    }

    pub(crate) fn start_typing(&self, channel: ChannelId) -> TypingSession {
        let messanger = self.messanger.as_ref().expect(
            "need messager; did you forget to call .set_messager(...) on robespierre::Context?",
        );

        messanger.send(ConnectionMessage::StartTyping { channel });

        TypingSession::new(channel, messanger.clone())
    }
}

impl robespierre_events::Context for Context {
    fn set_messanger(self, messanger: ConnectionMessanger) -> Self {
        Self {
            messanger: Some(messanger),
            ..self
        }
    }
}

impl HasCache for Context {
    fn get_cache(&self) -> Option<&Cache> {
        self.cache.as_ref().map(|it| &**it)
    }
}
