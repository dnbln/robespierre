use serde::{Deserialize, Serialize};

use bitflags::bitflags;

use crate::{
    attachments::Attachment,
    id::{AttachmentId, UserId},
};

/// An user
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct User {
    /// The user id.
    #[serde(rename = "_id")]
    pub id: UserId,
    /// The username
    pub username: String,
    /// The avatar (if available).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Attachment>,
    /// The relationships with other users.
    /// 
    /// Note: this is only available is this user
    /// is the currently logged-in user.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<Relationship>,
    /// The badges of the user
    // as they are currently undocumented, temporarely
    // represented with an u32;
    // TOOD: change to bitfield once documented
    // revolt.js uses a number too
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badges: Option<u32>,
    /// The current status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    /// The relationship the currently logged-in user has
    /// with this user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relationship: Option<RelationshipStatus>,
    /// Whether this user is online or not.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
    /// Refer to [`UserFlags`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<UserFlags>,
    /// If this is a bot, then [`BotInfo`] will
    /// contain the user id of the owner of the bot.
    /// If it is not a bot, then it will be [`None`]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot: Option<BotInfo>,
}

/// An user struct where all the fields are optional, and can be
/// used to update an [`User`] with the [`PartialUser::patch`]
/// function.
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
    /// Treats the [`PartialUser`] as a list of modifications
    /// that have to be applied to the given user, and applies
    /// them.
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

/// The relationship types
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

/// The status of an user.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct Status {
    /// Status text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// User presence, like online, invisible, idle, or busy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence: Option<UserPresence>,
}

/// User presence
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
    #[doc = "User flags"]
    pub struct UserFlags: u32 {
        const SUSPENDED = 0x1;
        const DELETED = 0x2;
        const BANNED = 0x4;
    }
}

/// Some information specific to bots.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(deny_unknown_fields)]
pub struct BotInfo {
    /// The id of the owner of the bot.
    pub owner: UserId,
}

/// A patch to the profile data of an user.
#[derive(Serialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct UserProfileDataPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<AttachmentId>,
}


/// A patch to an user.
#[derive(Serialize, Default, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[non_exhaustive]
pub struct UserEditPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<UserProfileDataPatch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<AttachmentId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<UserField>,
}

/// Profile data about an user.
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct UserProfileData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<Attachment>,
}

/// An user field.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum UserField {
    Avatar,
    ProfileBackground,
    ProfileContent,
    StatusText,
}

impl UserField {
    /// Removes this field from the user.
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
