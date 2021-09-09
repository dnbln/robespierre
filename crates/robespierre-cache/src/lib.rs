// TODO: documentation

use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    iter::FromIterator,
    sync::Arc,
};

use async_trait::async_trait;
use tokio::sync::RwLock;

use robespierre_models::{
    channel::{Channel, ChannelField, Message, PartialChannel, PartialMessage},
    events::ServerToClientEvent,
    id::{ChannelId, MemberId, MessageId, RoleId, ServerId, UserId},
    server::{
        Member, MemberField, PartialMember, PartialRole, PartialServer, Role, RoleField, Server,
        ServerField,
    },
    user::{PartialUser, User, UserField},
};

#[derive(Default)]
pub struct CacheConfig {
    /// number of messages to cache / channel, 0 for no caching
    pub messages: usize,
}

pub struct Cache {
    config: CacheConfig,

    users: RwLock<HashMap<UserId, User>>,
    servers: RwLock<HashMap<ServerId, Server>>,
    roles: RwLock<HashMap<RoleId, Role>>,
    members: RwLock<HashMap<MemberId, Member>>,
    channels: RwLock<HashMap<ChannelId, Channel>>,
    messages: RwLock<HashMap<ChannelId, HashMap<MessageId, Message>>>,
    message_queue: RwLock<HashMap<ChannelId, VecDeque<MessageId>>>,
}

impl Cache {
    pub fn new(config: CacheConfig) -> Arc<Self> {
        Arc::new(Self {
            config,

            users: RwLock::new(HashMap::new()),
            servers: RwLock::new(HashMap::new()),
            roles: RwLock::new(HashMap::new()),
            members: RwLock::new(HashMap::new()),
            channels: RwLock::new(HashMap::new()),
            messages: RwLock::new(HashMap::new()),
            message_queue: RwLock::new(HashMap::new()),
        })
    }
}

macro_rules! cache_field {
    ($id_ty:ty, $full_ty:ty, $cloner:ident, $get_data:ident, $field:ident) => {
        impl Cache {
            pub async fn $cloner(&self, id: $id_ty) -> Option<$full_ty> {
                self.$get_data(id, Clone::clone).await
            }

            pub async fn $get_data<F, T>(&self, id: $id_ty, f: F) -> Option<T>
            where
                F: FnOnce(&$full_ty) -> T,
            {
                self.$field.read().await.get(&id).map(f)
            }
        }
    };

    ($id_ty:ty, $full_ty:ty, $cloner:ident, $get_data:ident, $field:ident, $commit_function:ident, $key_field:ident) => {
        impl Cache {
            pub async fn $cloner(&self, id: $id_ty) -> Option<$full_ty> {
                self.$get_data(id, Clone::clone).await
            }

            pub async fn $get_data<F, T>(&self, id: $id_ty, f: F) -> Option<T>
            where
                F: FnOnce(&$full_ty) -> T,
            {
                self.$field.read().await.get(&id).map(f)
            }

            pub async fn $commit_function(&self, v: &$full_ty) {
                self.$field.write().await.insert(v.$key_field, v.clone());
            }
        }
    };
}

cache_field! {UserId, User, get_user, get_user_data, users, commit_user, id}

impl Cache {
    pub async fn patch_user(
        &self,
        user_id: UserId,
        patch: impl FnOnce() -> PartialUser,
        remove: Option<UserField>,
    ) {
        let mut lock = self.users.write().await;
        if let Some(user) = lock.get_mut(&user_id) {
            let patch = patch();

            patch.patch(user);
            if let Some(remove) = remove {
                remove.remove_patch(user);
            }
        }
    }
}

cache_field! {ServerId, Server, get_server, get_server_data, servers, commit_server, id}

impl Cache {
    pub async fn patch_server(
        &self,
        server_id: ServerId,
        patch: impl FnOnce() -> PartialServer,
        remove: Option<ServerField>,
    ) {
        let mut lock = self.servers.write().await;
        if let Some(server) = lock.get_mut(&server_id) {
            let patch = patch();

            patch.patch(server);
            if let Some(remove) = remove {
                remove.remove_patch(server);
            }
        }
    }
}

cache_field! {RoleId, Role, get_role, get_role_data, roles}

impl Cache {
    pub async fn commit_role(&self, role_id: RoleId, role: &Role) {
        let mut lock = self.roles.write().await;
        lock.insert(role_id, role.clone());
    }

    pub async fn patch_role(
        &self,
        role_id: RoleId,
        patch: impl FnOnce() -> PartialRole,
        remove: Option<RoleField>,
    ) {
        let mut lock = self.roles.write().await;
        if let Some(role) = lock.get_mut(&role_id) {
            let patch = patch();

            patch.patch(role);
            if let Some(remove) = remove {
                remove.remove_patch(role);
            }
        }
    }
}

cache_field! {MemberId, Member, get_member, get_member_data, members, commit_member, id}

impl Cache {
    pub async fn patch_member(
        &self,
        member_id: MemberId,
        patch: impl FnOnce() -> PartialMember,
        remove: Option<MemberField>,
    ) {
        let mut lock = self.members.write().await;
        if let Some(member) = lock.get_mut(&member_id) {
            let patch = patch();

            patch.patch(member);
            if let Some(remove) = remove {
                remove.remove_patch(member);
            }
        }
    }
}

cache_field! {ChannelId, Channel, get_channel, get_channel_data, channels}

impl Cache {
    pub async fn commit_channel(&self, channel: &Channel) {
        self.channels
            .write()
            .await
            .insert(channel.id(), channel.clone());
    }

    pub async fn patch_channel(
        &self,
        channel_id: ChannelId,
        patch: impl FnOnce() -> PartialChannel,
        remove: Option<ChannelField>,
    ) {
        let mut lock = self.channels.write().await;
        if let Some(channel) = lock.get_mut(&channel_id) {
            let patch = patch();

            patch.patch(channel);
            if let Some(remove) = remove {
                remove.remove_patch(channel);
            }
        }
    }
}

impl Cache {
    pub async fn get_message(&self, channel: ChannelId, message: MessageId) -> Option<Message> {
        self.get_message_data(channel, message, Clone::clone).await
    }

    pub async fn get_message_data<F, T>(
        &self,
        channel: ChannelId,
        message: MessageId,
        f: F,
    ) -> Option<T>
    where
        F: FnOnce(&Message) -> T,
    {
        self.messages
            .read()
            .await
            .get(&channel)?
            .get(&message)
            .map(f)
    }

    pub async fn commit_message(&self, message: &Message) {
        if self.config.messages == 0 {
            return;
        }

        let mut queue_lock = self.message_queue.write().await;
        let deque = queue_lock.entry(message.channel).or_insert_with(VecDeque::new);

        match self.messages.write().await.entry(message.channel) {
            Entry::Occupied(mut m) => {
                m.get_mut().insert(message.id, message.clone());

                deque.push_back(message.id);

                if deque.len() > self.config.messages {
                    if let Some(oldest) = deque.pop_front() {
                        m.get_mut().remove(&oldest);
                    }
                }
            }
            Entry::Vacant(v) => {
                deque.push_back(message.id);
                v.insert(HashMap::from_iter([(message.id, message.clone())]));
            }
        }
    }

    pub async fn patch_message(
        &self,
        channel_id: ChannelId,
        message_id: MessageId,
        patch: impl FnOnce() -> PartialMessage,
    ) {
        let mut lock = self.messages.write().await;
        if let Some(ch) = lock.get_mut(&channel_id) {
            if let Some(message) = ch.get_mut(&message_id) {
                let patch = patch();

                patch.patch(message);
            }
        }
    }
}

pub trait HasCache: Send + Sync {
    fn get_cache(&self) -> Option<&Cache>;
}

impl HasCache for Cache {
    fn get_cache(&self) -> Option<&Cache> {
        Some(self)
    }
}

impl HasCache for Arc<Cache> {
    fn get_cache(&self) -> Option<&Cache> {
        Some(self)
    }
}

#[async_trait]
pub trait CommitToCache: Send + Sync {
    async fn commit_to_cache<C: HasCache>(self, c: &C) -> Self
    where
        Self: Sized,
    {
        self.commit_to_cache_ref(c).await;

        self
    }

    async fn commit_to_cache_ref<C: HasCache>(&self, c: &C) {
        if let Some(c) = c.get_cache() {
            Self::__commit_to_cache(self, c).await;
        }
    }

    async fn __commit_to_cache(&self, cache: &Cache);
}

#[async_trait]
impl CommitToCache for User {
    async fn __commit_to_cache(&self, cache: &Cache) {
        cache.commit_user(self).await;
    }
}

#[async_trait]
impl CommitToCache for Channel {
    async fn __commit_to_cache(&self, cache: &Cache) {
        cache.commit_channel(self).await;
    }
}

#[async_trait]
impl CommitToCache for Server {
    async fn __commit_to_cache(&self, cache: &Cache) {
        cache.commit_server(self).await;
    }
}

#[async_trait]
impl CommitToCache for Member {
    async fn __commit_to_cache(&self, cache: &Cache) {
        cache.commit_member(self).await;
    }
}

#[async_trait]
impl CommitToCache for Message {
    async fn __commit_to_cache(&self, cache: &Cache) {
        cache.commit_message(self).await;
    }
}

#[async_trait]
impl<'a> CommitToCache for (RoleId, &'a Role) {
    async fn __commit_to_cache(&self, cache: &Cache) {
        cache.commit_role(self.0, self.1).await
    }
}

#[async_trait]
impl CommitToCache for ServerToClientEvent {
    async fn __commit_to_cache(&self, cache: &Cache) {
        #[allow(unused_variables)]
        match self {
            ServerToClientEvent::Error { .. } => {}
            ServerToClientEvent::Authenticated => {}
            ServerToClientEvent::Pong { .. } => {}
            ServerToClientEvent::Ready { event } => {
                for user in event.users.iter() {
                    user.commit_to_cache_ref(cache).await;
                }
                for channel in event.channels.iter() {
                    channel.commit_to_cache_ref(cache).await;
                }
                for server in event.servers.iter() {
                    server.commit_to_cache_ref(cache).await;
                }
                for member in event.members.iter() {
                    member.commit_to_cache_ref(cache).await;
                }
            }
            ServerToClientEvent::Message { message } => {
                message.commit_to_cache_ref(cache).await;
            }
            ServerToClientEvent::MessageUpdate { id, channel, data } => {
                cache.patch_message(*channel, *id, || data.clone()).await;
            }
            ServerToClientEvent::MessageDelete { id, channel } => {}
            ServerToClientEvent::ChannelCreate { channel } => {}
            ServerToClientEvent::ChannelUpdate { id, data, clear } => {
                cache.patch_channel(*id, || data.clone(), *clear).await;
            }
            ServerToClientEvent::ChannelDelete { id } => {}
            ServerToClientEvent::ChannelGroupJoin { id, user } => {}
            ServerToClientEvent::ChannelGroupLeave { id, user } => {}
            ServerToClientEvent::ChannelStartTyping { id, user } => {}
            ServerToClientEvent::ChannelStopTyping { id, user } => {}
            ServerToClientEvent::ChannelAck {
                id,
                user,
                message_id,
            } => {}
            ServerToClientEvent::ServerUpdate { id, data, clear } => {
                cache.patch_server(*id, || data.clone(), *clear).await;
            }
            ServerToClientEvent::ServerDelete { id } => {}
            ServerToClientEvent::ServerMemberUpdate { id, data, clear } => {
                cache.patch_member(*id, || data.clone(), *clear).await;
            }
            ServerToClientEvent::ServerMemberJoin { id, user } => {}
            ServerToClientEvent::ServerMemberLeave { id, user } => {}
            ServerToClientEvent::ServerRoleUpdate {
                id,
                role_id,
                data,
                clear,
            } => {}
            ServerToClientEvent::ServerRoleDelete { id, role_id } => {}
            ServerToClientEvent::UserUpdate { id, data, clear } => {
                cache.patch_user(*id, || data.clone(), *clear).await;
            }
            ServerToClientEvent::UserRelationship { id, user, status } => {}
        }
    }
}
