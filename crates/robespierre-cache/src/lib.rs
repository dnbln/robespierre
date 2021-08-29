use std::collections::HashMap;

use tokio::sync::RwLock;

use robespierre_models::{channel::{Channel, Message}, id::{ChannelId, MemberId, MessageId, RoleId, ServerId, UserId}, server::{Member, Role, Server}, user::User};

#[derive(Default)]
pub struct CacheConfig {
    /// number of messages to cache / channel, 0 for no caching
    pub messages: usize,
}

pub struct Cache {
    users: RwLock<HashMap<UserId, User>>,
    servers: RwLock<HashMap<ServerId, Server>>,
    roles: RwLock<HashMap<RoleId, Role>>,
    members: RwLock<HashMap<MemberId, Member>>,
    channels: RwLock<HashMap<ChannelId, Channel>>,
    messages: RwLock<HashMap<ChannelId, HashMap<MessageId, Message>>>,
}

macro_rules! cache_field {
    ($id_ty:ty, $full_ty:ty, $cloner:ident, $get_data:ident, $field:ident) => {
        impl Cache {
            pub async fn $cloner(&self, id: &$id_ty) -> Option<$full_ty> {
                self.$get_data(id, Clone::clone).await
            }

            pub async fn $get_data<F, T>(&self, id: &$id_ty, f: F) -> Option<T>
            where
                F: FnOnce(&$full_ty) -> T,
            {
                self.$field.read().await.get(id).map(f)
            }
        }
    };
}

cache_field! {UserId, User, get_user, get_user_data, users}
cache_field! {ServerId, Server, get_server, get_server_data, servers}
cache_field! {RoleId, Role, get_role, get_role_data, roles}
cache_field! {MemberId, Member, get_member, get_member_data, members}
cache_field! {ChannelId, Channel, get_channel, get_channel_data, channels}
