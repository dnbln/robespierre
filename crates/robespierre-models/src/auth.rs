use serde::{Deserialize, Serialize};

use crate::id::{SessionId, UserId};

/*
Newtypes
*/

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct Token(pub String);

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct WebPushSubscription(pub String);

/*
Types
*/

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Auth.ts#L3-L13
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct Account {
    #[serde(rename = "_id")]
    pub id: UserId,
    pub email: String,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Auth.ts#L15-L40
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct Session {
    #[serde(rename = "_id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<SessionId>,
    pub user_id: UserId,
    pub token: Token,
    #[serde(rename = "name")]
    pub device_name: String,
    #[serde(
        rename = "subscription",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub subscription: Option<WebPushSubscription>,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Auth.ts#L42-L52
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct SessionInfo {
    #[serde(rename = "_id")]
    id: SessionId,
    #[serde(rename = "name")]
    device_name: String,
}
