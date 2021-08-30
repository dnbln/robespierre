use serde::{Deserialize, Serialize};

use bitflags::bitflags;

use crate::{
    attachments::{Attachment, AutumnFileId},
    id::UserId,
};

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: UserId,
    pub username: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Attachment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<Relationship>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badges: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relationship: Option<RelationshipStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<UserFlags>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot: Option<BotInfo>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct PartialUser {
    #[serde(rename = "_id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<UserId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Attachment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relations: Option<Vec<Relationship>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badges: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relationship: Option<RelationshipStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<UserFlags>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot: Option<BotInfo>,
}

impl PartialUser {
    pub fn patch(self, user: &mut User) {
        let PartialUser {
            id: pid,
            username: pusername,
            avatar: pavatar,
            relations: prelations,
            badges: pbadges,
            status: pstatus,
            relationship: prelationship,
            online: ponline,
            flags: pflags,
            bot: pbot,
        } = self;
        let User {
            id,
            username,
            avatar,
            relations,
            badges,
            status,
            relationship,
            online,
            flags,
            bot,
        } = user;

        if let Some(pid) = pid {
            *id = pid;
        }
        if let Some(pusername) = pusername {
            *username = pusername;
        }
        if let Some(pavatar) = pavatar {
            *avatar = Some(pavatar);
        }
        if let Some(prelations) = prelations {
            *relations = prelations;
        }
        if let Some(pbadges) = pbadges {
            *badges = Some(pbadges);
        }
        if let Some(pstatus) = pstatus {
            *status = Some(pstatus);
        }
        if let Some(prelationship) = prelationship {
            *relationship = Some(prelationship);
        }
        if let Some(ponline) = ponline {
            *online = Some(ponline);
        }
        if let Some(pflags) = pflags {
            *flags = Some(pflags);
        }
        if let Some(pbot) = pbot {
            *bot = Some(pbot);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct Relationship {
    pub status: RelationshipStatus,
    #[serde(rename = "_id")]
    pub id: UserId,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub enum RelationshipStatus {
    Blocked,
    BlockedOther,
    Friend,
    Incoming,
    None,
    Outgoing,
    User,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct Status {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence: Option<UserPresence>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub enum UserPresence {
    Busy,
    Idle,
    Invisible,
    Online,
}

bitflags! {
    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct UserFlags: u32 {
        const SUSPENDED = 0x1;
        const DELETED = 0x2;
        const BANNED = 0x4;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct BotInfo {
    pub owner: UserId,
}

#[derive(Serialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct UserProfileDataPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<AutumnFileId>,
}

#[derive(Serialize, Default, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[non_exhaustive]
pub struct UserEditPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<UserProfileDataPatch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<AutumnFileId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<UserField>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct UserProfileData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<Attachment>,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum UserField {
    Avatar,
    ProfileBackground,
    ProfileContent,
    StatusText,
}

impl UserField {
    pub fn remove_patch(self, user: &mut User) {
        match self {
            Self::Avatar => user.avatar = None,
            Self::ProfileBackground => {}
            Self::ProfileContent => {}
            Self::StatusText => {
                if let Some(Status { text, .. }) = &mut user.status {
                    *text = None
                }
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct NewRelationshipResponse {
    pub status: RelationshipStatus,
}
