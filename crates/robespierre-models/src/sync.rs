use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::id::{ChannelId, MessageId, UserId};

/*
Types
*/

// https://github.com/revoltchat/api/blob/094f8e650dbbbfd6a61be60d20943ea471a816c6/types/Sync.ts#L3-L5

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct UserSettings(pub HashMap<String, SettingTuple>);

// https://github.com/revoltchat/api/blob/094f8e650dbbbfd6a61be60d20943ea471a816c6/types/Sync.ts#L7-L10

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ChannelCompositeKey {
    pub channel: ChannelId,
    pub user: UserId,
}

// https://github.com/revoltchat/api/blob/094f8e650dbbbfd6a61be60d20943ea471a816c6/types/Sync.ts#L12-L17

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ChannelUnread {
    #[serde(rename = "_id")]
    pub id: ChannelCompositeKey,
    pub last_id: MessageId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mentions: Vec<MessageId>,
}

// https://github.com/revoltchat/api/blob/094f8e650dbbbfd6a61be60d20943ea471a816c6/types/Sync.ts#L19-L23

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct WebPushSubscription {
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}

/*
Extra
*/

// https://github.com/revoltchat/api/blob/094f8e650dbbbfd6a61be60d20943ea471a816c6/types/Sync.ts#L4

pub type SettingTuple = (usize, String);