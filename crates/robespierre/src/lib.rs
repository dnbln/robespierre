use std::sync::Arc;

use robespierre_cache::Cache;
use robespierre_events::{RawEventHandler, ReadyEvent, ServerToClientEvent};
use robespierre_http::Http;
use robespierre_models::{
    channel::{Channel, ChannelField, Message, PartialChannel, PartialMessage},
    id::{ChannelId, MessageId, RoleId, ServerId, UserId},
    server::{MemberField, PartialMember, PartialRole, PartialServer, RoleField, ServerField},
    user::{PartialUser, RelationshipStatus, UserField},
};

pub use async_trait::async_trait;

#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait EventHandler: Send + Sync {
    async fn on_ready(&self, ctx: Context, ready: ReadyEvent) {}
    async fn on_message(&self, ctx: Context, message: Message) {}
    async fn on_message_update(&self, ctx: Context, id: MessageId, modifications: PartialMessage) {}
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
        server: ServerId,
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
            ServerToClientEvent::MessageUpdate { id, data } => {
                self.0.on_message_update(ctx, id, data).await
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
}
