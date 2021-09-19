use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use crate::{
    autumn::{Attachment, AttachmentId},
    id::UserId,
};

/*
Types
*/

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Users.ts#L4-L10

/// Username
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct Username(pub String);

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Users.ts#L12-L23

/// Your relationship with the user
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RelationshipStatus {
    None,
    User,
    Friend,
    Outgoing,
    Incoming,
    Blocked,
    BlockedOther,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Users.ts#L25-L27
// pub struct RelationshipOnly {
//     status: RelationshipStatus
// }

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Users.ts#L29-L34

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct Relationship {
    pub status: RelationshipStatus,
    /// Other user's ID
    #[serde(rename = "_id")]
    pub id: UserId,
}

// https://github.com/revoltchat/api/blob/master/types/Users.ts#L36-L44

/// User presence
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub enum UserPresence {
    Online,
    Idle,
    Busy,
    Invisible,
}

// https://github.com/revoltchat/api/blob/master/types/Users.ts#L46-L58

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct Status {
    /// Status text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// User presence, like online, invisible, idle, or busy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence: Option<UserPresence>,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Users.ts#L60-L67

bitflags! {
    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    #[doc = "User badges"]
    pub struct Badges: u32 {
        const DEVELOPER = 1;
        const TRANSLATOR = 2;
        const SUPPORTER = 4;
        const RESPONSIBLE_DISCLOSURE = 8;
        const REVOLT_TEAM = 16;
        const EARLY_ADOPTER = 256;
    }
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Users.ts#L69-L77

/// Bot information
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct BotInformation {
    /// The User ID of the owner of this bot
    pub owner: UserId,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Users.ts#L79-L128

/// An user
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct User {
    /// The user id.
    #[serde(rename = "_id")]
    pub id: UserId,
    /// The username
    pub username: Username,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badges: Option<Badges>,
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
    pub bot: Option<BotInformation>,

    /// Profile data
    /// Sometimes the server sends it although fetching
    /// an user doesn't retrieve the profile data too.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<Profile>,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Users.ts#L115-L122

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

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Users.ts#L130-L142

/// Profile data about an user.
#[derive(Serialize, Deserialize, Default, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct Profile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<Attachment>,
}

/*
Extra
*/

/// An user struct where all the fields are optional, and can be
/// used to update an [`User`] with the [`PartialUser::patch`]
/// function.
#[derive(Serialize, Deserialize, Default, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct UserPatch {
    #[serde(rename = "_id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<UserId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<Username>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Attachment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relations: Option<Vec<Relationship>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badges: Option<Badges>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relationship: Option<RelationshipStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<UserFlags>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bot: Option<BotInformation>,
    #[serde(
        rename = "profile.content",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub profile_content: Option<String>,
    #[serde(
        rename = "profile.background",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub profile_background: Option<Attachment>,
}

impl UserPatch {
    /// Treats the [`PartialUser`] as a list of modifications
    /// that have to be applied to the given user, and applies
    /// them.
    pub fn patch(self, user: &mut User) {
        let UserPatch {
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
            profile_content: pprofile_content,
            profile_background: pprofile_background,
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
            profile,
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
        if let Some(pprofile_content) = pprofile_content {
            profile.get_or_insert(Default::default()).content = Some(pprofile_content);
        }
        if let Some(pprofile_background) = pprofile_background {
            profile.get_or_insert(Default::default()).background = Some(pprofile_background);
        }
    }
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
