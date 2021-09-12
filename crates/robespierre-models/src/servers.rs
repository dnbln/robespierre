use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    autumn::Attachment,
    channels::ChannelPermissions,
    id::{ChannelId, MemberId, RoleId, ServerId, UserId},
};

/*
Types
*/

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Servers.ts#L4-L8

pub type MemberCompositeKey = MemberId;

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Servers.ts#L10-L18

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Member {
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Attachment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<RoleId>,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Servers.ts#L20-L23

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Ban {
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Servers.ts#L25-L31

pub type PermissionTuple = (ServerPermissions, ChannelPermissions);

bitflags::bitflags! {
    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    #[doc = "Server permissions"]
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

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Servers.ts#L33-L46

pub type Color = String;

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Servers.ts#L48-L71

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Role {
    /// The name of the role.
    pub name: String,
    /// The permissions the role has.
    pub permissions: PermissionTuple,
    /// The color
    /// "Valid html color"
    /// - documentation
    ///
    /// The documentation says that this is untrusted input,
    /// and should not be inserted anywhere.
    /// Example usage:
    /// ```js
    /// document.body.style.color = role.color;
    /// ```
    #[serde(rename = "colour", default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    /// Whether this role is hoisted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hoist: Option<bool>,
    /// The rank of this role.
    ///
    /// The higher the rank, the lower in the role hierarchy it will be.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<usize>,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Servers.ts#L73

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct RoleInformation {
    pub name: String,
    #[serde(rename = "colour", default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hoist: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<usize>,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Servers.ts#L75-L81

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Category {
    pub id: ChannelId,
    pub title: String,
    pub channels: Vec<ChannelId>,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Servers.ts#L83-L92

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct SystemMessageChannels {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_joined: Option<ChannelId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_left: Option<ChannelId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_kicked: Option<ChannelId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_banned: Option<ChannelId>,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Servers.ts#L94-L160

/// A server.
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
    pub categories: Vec<Category>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_messages: Option<SystemMessageChannels>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<RolesObject>,
    pub default_permissions: PermissionTuple,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<Attachment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner: Option<Attachment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
}

/*
Extra
*/

/// A member where all the fields are optional, and can be treated as
/// a patch that can be applied to a [`Member`].
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

impl PartialMember {
    /// Treat self as a patch and apply it to member.
    pub fn patch(self, member: &mut Member) {
        let PartialMember {
            id: pid,
            nickname: pnickname,
            avatar: pavatar,
            roles: proles,
        } = self;
        let Member {
            id,
            nickname,
            avatar,
            roles,
        } = member;

        if let Some(pid) = pid {
            *id = pid;
        }
        if let Some(pnickname) = pnickname {
            *nickname = Some(pnickname);
        }
        if let Some(pavatar) = pavatar {
            *avatar = Some(pavatar);
        }
        if let Some(proles) = proles {
            *roles = proles;
        }
    }
}

/// A member field, that can be used to unset a field in a [`Member`].
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum MemberField {
    Nickname,
    Avatar,
}

impl MemberField {
    /// Treats self as a patch and removes the field from the member.
    pub fn remove_patch(self, member: &mut Member) {
        match self {
            Self::Nickname => member.nickname = None,
            Self::Avatar => member.avatar = None,
        }
    }
}

/// A "roles object", as a map of (key=[`RoleId`], value=[`Role`]) elements,
/// as that is how roles are represented within a [`Server`] object.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(transparent)]
pub struct RolesObject(HashMap<RoleId, Role>);

impl RolesObject {
    pub fn iter(&self) -> RolesIter {
        RolesIter(self.0.iter())
    }

    pub fn get(&self, id: &RoleId) -> Option<&Role> {
        self.0.get(id)
    }

    pub fn patch_role(&mut self, role_id: &RoleId, patch: PartialRole, remove: Option<RoleField>) {
        if let Some(refr) = self.0.get_mut(role_id) {
            patch.patch(refr);

            if let Some(remove) = remove {
                remove.remove_patch(refr);
            }
        }
    }

    pub fn remove(&mut self, id: &RoleId) {
        self.0.remove(id);
    }
}

pub struct RolesIter<'a>(std::collections::hash_map::Iter<'a, RoleId, Role>);

impl<'a> Iterator for RolesIter<'a> {
    type Item = (&'a RoleId, &'a Role);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// A role where all the fields are optional, and can be used to
/// describe a patch applied to a role.
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
    pub rank: Option<usize>,
}

impl PartialRole {
    /// Treat self as a patch and apply it to role.
    pub fn patch(self, role: &mut Role) {
        let PartialRole {
            name: pname,
            permissions: ppermissions,
            color: pcolor,
            hoist: phoist,
            rank: prank,
        } = self;
        let Role {
            name,
            permissions,
            color,
            hoist,
            rank,
        } = role;

        if let Some(pname) = pname {
            *name = pname;
        }
        if let Some(ppermissions) = ppermissions {
            *permissions = ppermissions;
        }
        if let Some(pcolor) = pcolor {
            *color = Some(pcolor);
        }
        if let Some(phoist) = phoist {
            *hoist = Some(phoist);
        }
        if let Some(prank) = prank {
            *rank = Some(prank);
        }
    }
}

/// A role field
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum RoleField {
    #[serde(rename = "Colour")]
    Color,
}

impl RoleField {
    /// Treats this role as a patch and removes the field from the role.
    pub fn remove_patch(self, role: &mut Role) {
        match self {
            Self::Color => role.color = None,
        }
    }
}

/// A server where all the fields are optional, and so can be
/// treated as a patch that can be applied to a [`Server`].
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
    pub categories: Option<Vec<Category>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_messages: Option<SystemMessageChannels>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<RolesObject>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_permissions: Option<(ServerPermissions, ChannelPermissions)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<Attachment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner: Option<Attachment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
}

impl PartialServer {
    /// Treats self as a patch and applies it to server.
    pub fn patch(self, serv: &mut Server) {
        let PartialServer {
            id: pid,
            nonce: pnonce,
            owner: powner,
            name: pname,
            description: pdescription,
            channels: pchannels,
            categories: pcategories,
            system_messages: psystem_messages,
            roles: proles,
            default_permissions: pdefault_permissions,
            icon: picon,
            banner: pbanner,
            nsfw: pnsfw,
        } = self;
        let Server {
            id,
            nonce,
            owner,
            name,
            description,
            channels,
            categories,
            system_messages,
            roles,
            default_permissions,
            icon,
            banner,
            nsfw,
        } = serv;

        if let Some(pid) = pid {
            *id = pid;
        }
        if let Some(pnonce) = pnonce {
            *nonce = Some(pnonce);
        }
        if let Some(powner) = powner {
            *owner = powner;
        }
        if let Some(pname) = pname {
            *name = pname;
        }
        if let Some(pdescription) = pdescription {
            *description = Some(pdescription);
        }
        if let Some(pchannels) = pchannels {
            *channels = pchannels;
        }
        if let Some(pcategories) = pcategories {
            *categories = pcategories;
        }
        if let Some(psystem_messages) = psystem_messages {
            *system_messages = Some(psystem_messages);
        }
        if let Some(proles) = proles {
            *roles = Some(proles);
        }
        if let Some(pdefault_permissions) = pdefault_permissions {
            *default_permissions = pdefault_permissions;
        }
        if let Some(picon) = picon {
            *icon = Some(picon);
        }
        if let Some(pbanner) = pbanner {
            *banner = Some(pbanner);
        }
        if let Some(pnsfw) = pnsfw {
            *nsfw = Some(pnsfw);
        }
    }
}

/// A server field, that can be used to unset a field in a [`Server`].
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum ServerField {
    Icon,
    Banner,
    Description,
}

impl ServerField {
    /// Treats self as a patch and removes the field from the server.
    pub fn remove_patch(self, server: &mut Server) {
        match self {
            Self::Icon => server.icon = None,
            Self::Banner => server.banner = None,
            Self::Description => server.description = None,
        }
    }
}
