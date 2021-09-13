use serde::{Deserialize, Serialize};

use crate::{autumn::Attachment, id::UserId, users::Username};

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Bots.ts#L5-L34

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Bot {
    /// Bot ID, matches bot's User ID
    #[serde(rename = "_id")]
    pub id: UserId,
    /// Bot owner's User ID
    pub owner: UserId,
    /// Bot authentication token.
    pub token: String,
    /// Whether the bot can be added by anyone.
    pub public: bool,

    /**
    Interactions endpoint URL

    Required for dynamic interactions such as bot commands and message actions. Events will be sent over HTTP and a response may be generated directly.
    */
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interactions_url: Option<String>,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Bots.ts#L36-L56

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct PublicBot {
    /// Bot ID, matches bot's User ID
    #[serde(rename = "_id")]
    id: UserId,

    /// Bot username
    username: Username,

    /// Bot avatar
    #[serde(default, skip_serializing_if = "Option::is_none")]
    avatar: Option<Attachment>,

    /// Bot description, taken from profile text
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

/*
Extra
*/
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BotField {
    InteractionsURL,
}
