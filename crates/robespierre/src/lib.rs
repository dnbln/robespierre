use std::sync::Arc;

use robespierre_cache::{Cache, CacheConfig, CommitToCache, HasCache};
use robespierre_events::{EventsError, RawEventHandler, ReadyEvent, ServerToClientEvent};
use robespierre_http::{Http, HttpError};
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

#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait EventHandler: Send + Sync {
    async fn on_ready(&self, ctx: Context, ready: ReadyEvent) {}
    async fn on_message(&self, ctx: Context, message: Message) {}
    async fn on_message_update(
        &self,
        ctx: Context,
        channel: ChannelId,
        message: MessageId,
        modifications: PartialMessage,
    ) {
    }
    async fn on_message_delete(&self, ctx: Context, channel_id: ChannelId, message_id: MessageId) {}
    async fn on_channel_create(&self, ctx: Context, channel: Channel) {}
    async fn on_channel_update(
        &self,
        ctx: Context,
        channel_id: ChannelId,
        modifications: PartialChannel,
        remove: Option<ChannelField>,
    ) {
    }
    async fn on_channel_delete(&self, ctx: Context, channel_id: ChannelId) {}
    async fn on_group_join(&self, ctx: Context, id: ChannelId, user: UserId) {}
    async fn on_group_leave(&self, ctx: Context, id: ChannelId, user: UserId) {}
    async fn on_start_typing(&self, ctx: Context, channel: ChannelId, user: UserId) {}
    async fn on_stop_typing(&self, ctx: Context, channel: ChannelId, user: UserId) {}
    async fn on_server_update(
        &self,
        ctx: Context,
        server: ServerId,
        modifications: PartialServer,
        remove: Option<ServerField>,
    ) {
    }
    async fn on_server_delete(&self, ctx: Context, server: ServerId) {}
    async fn on_server_member_join(&self, ctx: Context, server: ServerId, user: UserId) {}
    async fn on_server_member_update(
        &self,
        ctx: Context,
        member: MemberId,
        modifications: PartialMember,
        remove: Option<MemberField>,
    ) {
    }
    async fn on_server_member_leave(&self, ctx: Context, server: ServerId, user: UserId) {}
    async fn on_server_role_update(
        &self,
        ctx: Context,
        server: ServerId,
        role: RoleId,
        modifications: PartialRole,
        remove: Option<RoleField>,
    ) {
    }
    async fn on_server_role_delete(&self, ctx: Context, server: ServerId, role: RoleId) {}
    async fn on_user_update(
        &self,
        ctx: Context,
        id: UserId,
        modifications: PartialUser,
        remove: Option<UserField>,
    ) {
    }
    async fn on_user_relationship_update(
        &self,
        ctx: Context,
        self_id: UserId,
        other_id: UserId,
        status: RelationshipStatus,
    ) {
    }
}

#[derive(Clone)]
pub struct CacheWrap<T: EventHandler + Clone + 'static>(T);

impl<T> CacheWrap<T>
where
    T: EventHandler + Clone + 'static,
{
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
}

impl Context {
    pub fn new(http: Http) -> Self {
        Self {
            http: Arc::new(http),
            cache: None,
        }
    }

    pub fn with_cache(self, cache_config: CacheConfig) -> Self {
        Self {
            cache: Some(Cache::new(cache_config)),
            ..self
        }
    }
}

impl HasCache for Context {
    fn get_cache(&self) -> Option<&Cache> {
        self.cache.as_ref().map(|it| &**it)
    }
}
