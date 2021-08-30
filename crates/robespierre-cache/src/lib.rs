use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;

use robespierre_models::{
    channel::{Channel, Message},
    id::{ChannelId, MemberId, MessageId, RoleId, ServerId, UserId},
    server::{Member, Role, Server},
    user::User,
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
cache_field! {ServerId, Server, get_server, get_server_data, servers, commit_server, id}
cache_field! {RoleId, Role, get_role, get_role_data, roles}
cache_field! {MemberId, Member, get_member, get_member_data, members, commit_member, id}
cache_field! {ChannelId, Channel, get_channel, get_channel_data, channels}

impl Cache {
    async fn commit_channel(&self, channel: &Channel) {
        self.channels
            .write()
            .await
            .insert(channel.id(), channel.clone());
    }
}

pub trait HasCache: Send + Sync {
    fn get_cache(&self) -> Option<&Cache>;
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
