use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{attachments::Attachment, channel::ChannelPermissions, id::{ChannelId, MemberId, RoleId, ServerId, UserId}};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Server {
    #[serde(rename = "_id")]
    pub id: ServerId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    pub owner: UserId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub channels: Vec<ChannelId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<ChannelCategory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_messages: Option<SystemMessagesChannels>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<RolesObject>,
    pub default_permissions: (ServerPermissions, ChannelPermissions),
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<Attachment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner: Option<Attachment>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq)]
pub struct PartialServer {
    #[serde(rename = "_id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<ServerId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<UserId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channels: Option<Vec<ChannelId>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<ChannelCategory>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_messages: Option<SystemMessagesChannels>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<RolesObject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_permissions: Option<(ServerPermissions, ChannelPermissions)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<Attachment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner: Option<Attachment>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ChannelCategory {
    pub id: ChannelId,
    pub title: String,
    pub channels: Vec<ChannelId>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct SystemMessagesChannels {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_joined: Option<ChannelId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_left: Option<ChannelId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_kicked: Option<ChannelId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_banned: Option<ChannelId>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(transparent)]
pub struct RolesObject(HashMap<RoleId, Role>);

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Role {
    pub name: String,
    pub permissions: (ServerPermissions, ChannelPermissions),
    #[serde(rename = "colour", default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default)]
    pub hoist: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct PartialRole {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<(ServerPermissions, ChannelPermissions)>,
    #[serde(rename = "colour", default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hoist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum RoleField {
    #[serde(rename = "Colour")]
    Color,
}

bitflags::bitflags! {
    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct ServerPermissions: u32 {
        const VIEW = 0b00000000000000000000000000000001;            // 1
        const MANAGE_ROLES = 0b00000000000000000000000000000010;   // 2
        const MANAGE_CHANNELS = 0b00000000000000000000000000000100;  // 4
        const MANAGE_SERVER = 0b00000000000000000000000000001000;    // 8
        const KICK_MEMBERS = 0b00000000000000000000000000010000;     // 16
        const BAN_MEMBERS = 0b00000000000000000000000000100000;      // 32
        const CHANGE_NICKNAME = 0b00000000000000000001000000000000;  // 4096
        const MANAGE_NICKNAMES = 0b00000000000000000010000000000000; // 8192
        const CHANGE_AVATAR = 0b00000000000000000100000000000000;    // 16382
        const REMOVE_AVATARS = 0b00000000000000001000000000000000;   // 32768
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Member {
    #[serde(rename = "_id")]
    pub id: MemberId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Attachment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<RoleId>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct PartialMember {
    #[serde(rename = "_id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<MemberId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Attachment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<RoleId>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum ServerField {
    Icon, Banner, Description
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum MemberField {
    Nickname, Avatar
}
