use serde::{Deserialize, Serialize};

use crate::{
    autumn::Attachment,
    id::{ChannelId, InviteId, ServerId, UserId},
};

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Invites.ts#L4-L26

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ServerInvite {
    #[serde(rename = "_id")]
    pub id: InviteId,
    pub server: ServerId,
    pub creator: UserId,
    /// ID of the channel this invite is for.
    pub channel: ChannelId,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Invites.ts#L28

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(tag = "type")]
pub enum Invite {
    Invite(ServerInvite),
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Invites.ts#L30-L42

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct RetrievedInvite {
    pub server_id: ServerId,
    pub server_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_icon: Option<Attachment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_banner: Option<Attachment>,
    pub channel_id: ChannelId,
    pub channel_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_description: Option<String>,
    pub user_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_avatar: Option<Attachment>,
    pub member_count: usize,
}
