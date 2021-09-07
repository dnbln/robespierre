use std::{collections::HashMap, fmt::Display};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    attachments::Attachment,
    id::{AttachmentId, ChannelId, MessageId, RoleId, ServerId, UserId},
};

/// A channel
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "channel_type")]
pub enum Channel {
    SavedMessages {
        #[serde(rename = "_id")]
        id: ChannelId,
        user: UserId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        nonce: Option<String>,
    },
    DirectMessage {
        #[serde(rename = "_id")]
        id: ChannelId,
        recipients: Vec<UserId>,
        last_message: LastMessageData,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        nonce: Option<String>,
    },
    Group {
        #[serde(rename = "_id")]
        id: ChannelId,
        recipients: Vec<UserId>,
        name: String,
        owner: UserId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        last_message: LastMessageData,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        icon: Option<Attachment>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        permissions: Option<ChannelPermissions>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        nonce: Option<String>,
    },
    TextChannel {
        #[serde(rename = "_id")]
        id: ChannelId,
        server: ServerId,
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        icon: Option<Attachment>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        default_permissions: Option<ChannelPermissions>,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        role_permissions: HashMap<RoleId, ChannelPermissions>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        last_message: Option<MessageId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        nonce: Option<String>,
    },
    VoiceChannel {
        #[serde(rename = "_id")]
        id: ChannelId,
        server: ServerId,
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        icon: Option<Attachment>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        default_permissions: Option<ChannelPermissions>,
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        role_permissions: HashMap<RoleId, ChannelPermissions>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        nonce: Option<String>,
    },
}

impl Channel {
    pub fn id(&self) -> ChannelId {
        match self {
            Self::SavedMessages { id, .. } => *id,
            Self::DirectMessage { id, .. } => *id,
            Self::Group { id, .. } => *id,
            Self::TextChannel { id, .. } => *id,
            Self::VoiceChannel { id, .. } => *id,
        }
    }

    pub fn server_id(&self) -> Option<ServerId> {
        match self {
            Channel::TextChannel { server, .. } | Channel::VoiceChannel { server, .. } => {
                Some(*server)
            }
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum ChannelType {
    SavedMessages,
    DirectMessage,
    Group,
    TextChannel,
    VoiceChannel,
}

/// A channel where all the fields are optional, and can be treated as a patch that
/// can be applied to a [`Channel`].
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PartialChannel {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    user: Option<UserId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    nonce: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    recipients: Option<Vec<UserId>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_message: Option<LastMessageData>,

    name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<UserId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    icon: Option<Attachment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    permissions: Option<ChannelPermissions>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    server: Option<ServerId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default_permissions: Option<ChannelPermissions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    role_permissions: Option<HashMap<RoleId, ChannelPermissions>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    channel_type: Option<ChannelType>,
}

impl PartialChannel {
    /// Treats self as a patch and applies it to channel.
    pub fn patch(self, ch: &mut Channel) {
        match ch {
            Channel::SavedMessages { id: _, user, nonce } => {
                if let Some(puser) = self.user {
                    *user = puser;
                }
                if let Some(pnonce) = self.nonce {
                    *nonce = Some(pnonce);
                }
            }
            Channel::DirectMessage {
                id: _,
                recipients,
                last_message,
                nonce,
            } => {
                if let Some(precipients) = self.recipients {
                    *recipients = precipients;
                }
                if let Some(plast_message) = self.last_message {
                    *last_message = plast_message;
                }
                if let Some(pnonce) = self.nonce {
                    *nonce = Some(pnonce);
                }
            }
            Channel::Group {
                id: _,
                recipients,
                name,
                owner,
                description,
                last_message,
                icon,
                permissions,
                nonce,
            } => {
                if let Some(precipients) = self.recipients {
                    *recipients = precipients;
                }
                if let Some(pname) = self.name {
                    *name = pname;
                }
                if let Some(powner) = self.owner {
                    *owner = powner;
                }
                if let Some(pdescription) = self.description {
                    *description = Some(pdescription);
                }
                if let Some(plast_message) = self.last_message {
                    *last_message = plast_message;
                }
                if let Some(picon) = self.icon {
                    *icon = Some(picon);
                }
                if let Some(ppermissions) = self.permissions {
                    *permissions = Some(ppermissions);
                }
                if let Some(pnonce) = self.nonce {
                    *nonce = Some(pnonce);
                }
            }
            Channel::TextChannel {
                id: _,
                server,
                name,
                description,
                icon,
                default_permissions,
                role_permissions,
                last_message: _,
                nonce,
            } => {
                if let Some(pserver) = self.server {
                    *server = pserver;
                }
                if let Some(pname) = self.name {
                    *name = pname;
                }
                if let Some(pdescription) = self.description {
                    *description = Some(pdescription);
                }
                if let Some(picon) = self.icon {
                    *icon = Some(picon);
                }
                if let Some(pdefault_permissions) = self.default_permissions {
                    *default_permissions = Some(pdefault_permissions);
                }
                if let Some(prole_permissions) = self.role_permissions {
                    *role_permissions = prole_permissions;
                }
                // if let Some(plast_message) = self.last_message {
                //     *last_message = Some(plast_message);
                // }
                if let Some(pnonce) = self.nonce {
                    *nonce = Some(pnonce);
                }
            }
            Channel::VoiceChannel {
                id: _,
                server,
                name,
                description,
                icon,
                default_permissions,
                role_permissions,
                nonce,
            } => {
                if let Some(pserver) = self.server {
                    *server = pserver;
                }
                if let Some(pname) = self.name {
                    *name = pname;
                }
                if let Some(pdescription) = self.description {
                    *description = Some(pdescription);
                }
                if let Some(picon) = self.icon {
                    *icon = Some(picon);
                }
                if let Some(pdefault_permissions) = self.default_permissions {
                    *default_permissions = Some(pdefault_permissions);
                }
                if let Some(prole_permissions) = self.role_permissions {
                    *role_permissions = prole_permissions;
                }
                if let Some(pnonce) = self.nonce {
                    *nonce = Some(pnonce);
                }
            }
        }
    }
}

bitflags::bitflags! {
    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    #[doc = "Channel permissions"]
    pub struct ChannelPermissions: u32 {
        const VIEW = 0b00000000000000000000000000000001;           // 1
        const SEND_MESSAGE = 0b00000000000000000000000000000010;    // 2
        const MANAGE_MESSAGES = 0b00000000000000000000000000000100; // 4
        const MANAGE_CHANNEL = 0b00000000000000000000000000001000;  // 8
        const VOICE_CALL =  0b00000000000000000000000000010000;      // 16
        const INVITE_OTHERS = 0b00000000000000000000000000100000;   // 32
        const EMBED_LINKS = 0b00000000000000000000000001000000;   // 64
        const UPLOAD_FILES = 0b00000000000000000000000010000000;   // 128
    }
}

/// A dm channel or group
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "channel_type")]
pub enum DmChannel {
    DirectMessage {
        #[serde(rename = "_id")]
        id: ChannelId,
        recipients: Vec<UserId>,
        last_message: LastMessageData,

        #[serde(default, skip_serializing_if = "Option::is_none")]
        nonce: Option<String>,
    },
    Group {
        #[serde(rename = "_id")]
        id: ChannelId,
        recipients: Vec<UserId>,
        name: String,
        owner: UserId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        last_message: LastMessageData,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        icon: Option<Attachment>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        permissions: Option<ChannelPermissions>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        nonce: Option<String>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct LastMessageData {
    #[serde(rename = "_id")]
    pub id: MessageId,
    pub author: UserId,
    pub short: String,
}

/// Data about reply
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReplyData {
    pub id: MessageId,
    pub mention: bool,
}

/// A message
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: MessageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    pub channel: ChannelId,
    pub author: UserId,
    pub content: MessageContent,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited: Option<Date>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub embeds: Vec<Embed>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mentions: Vec<UserId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replies: Vec<MessageId>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SystemMessage {
    Text { content: String },
    UserAdded { id: UserId, by: UserId },
    UserRemove { id: UserId, by: UserId },
    UserJoined { id: UserId },
    UserLeft { id: UserId },
    UserKicked { id: UserId },
    UserBanned { id: UserId },
    ChannelRenamed { name: String, by: UserId },
    ChannelDescriptionChanged { by: UserId },
    ChannelIconChanged { by: UserId },
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(untagged)]
pub enum MessageContent {
    Content(String),
    SystemMessage(SystemMessage),
}

/// A message where all the fields are optional, and can be treated as a patch
/// that can be applied to a [`Message`].
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PartialMessage {
    #[serde(rename = "_id", default, skip_serializing_if = "Option::is_none")]
    pub id: Option<MessageId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<ChannelId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<UserId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited: Option<Date>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embeds: Option<Vec<Embed>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<UserId>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replies: Option<Vec<MessageId>>,
}

impl PartialMessage {
    /// Treats self as a patch and applies it to message.
    pub fn patch(self, m: &mut Message) {
        let PartialMessage {
            id: pid,
            nonce: pnonce,
            channel: pchannel,
            author: pauthor,
            content: pcontent,
            attachments: pattachments,
            edited: pedited,
            embeds: pembeds,
            mentions: pmentions,
            replies: preplies,
        } = self;
        let Message {
            id,
            nonce,
            channel,
            author,
            content,
            attachments,
            edited,
            embeds,
            mentions,
            replies,
        } = m;

        if let Some(pid) = pid {
            *id = pid;
        }
        if let Some(pnonce) = pnonce {
            *nonce = Some(pnonce);
        }
        if let Some(pchannel) = pchannel {
            *channel = pchannel;
        }
        if let Some(pauthor) = pauthor {
            *author = pauthor;
        }
        if let Some(pcontent) = pcontent {
            *content = pcontent;
        }
        if let Some(pattachments) = pattachments {
            *attachments = pattachments;
        }
        if let Some(pedited) = pedited {
            *edited = Some(pedited);
        }
        if let Some(pembeds) = pembeds {
            *embeds = pembeds;
        }
        if let Some(pmentions) = pmentions {
            *mentions = pmentions;
        }
        if let Some(preplies) = preplies {
            *replies = preplies;
        }
    }
}

/// Helper to serialize / deserialize mongo dates
#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(from = "WrappedDate", into = "WrappedDate")]
pub struct Date(pub DateTime<Utc>);

impl From<Date> for WrappedDate {
    fn from(d: Date) -> Self {
        Self { date: d.0 }
    }
}

impl From<WrappedDate> for Date {
    fn from(d: WrappedDate) -> Self {
        Self(d.date)
    }
}

#[derive(Serialize, Deserialize)]
struct WrappedDate {
    #[serde(rename = "$date")]
    date: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "type")]
pub enum Embed {
    None,
    Website {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        special: Option<SpecialWebsiteEmbed>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        image: Option<EmbeddedImage>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        video: Option<EmbeddedVideo>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        site_name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        icon_url: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        color: Option<String>,
    },
    Image {
        url: String,
        width: u32,
        height: u32,
        size: SizeType,
    },
}

/// Data about an embed of a special website, if it is the case
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(tag = "type")]
pub enum SpecialWebsiteEmbed {
    None,
    YouTube {
        id: String,
    },
    Twitch {
        content_type: TwitchContentType,
        id: String,
    },
    Spotify {
        content_type: String,
        id: String,
    },
    Soundcloud,
    Bandcamp {
        content_type: BandcampContentType,
        id: String,
    },
}

/// Twich content type
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum TwitchContentType {
    Channel,
    Clip,
    Video,
}

/// Bandcamp content type
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum BandcampContentType {
    Album,
    Track,
}

/// Embedded image
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct EmbeddedImage {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub size: SizeType,
}

/// Embedded video
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct EmbeddedVideo {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

/// Size type
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum SizeType {
    Large,
    Preview,
}

/// A patch to a channel
#[derive(Serialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct ChannelEditPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<AttachmentId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<ChannelField>,
}

/// A channel field that can be removed from a channel
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum ChannelField {
    Description,
    Icon,
}

impl ChannelField {
    /// Treats self as a patch and removes the field from the channel.
    pub fn remove_patch(self, channel: &mut Channel) {
        match self {
            Self::Description => match channel {
                Channel::Group { description, .. }
                | Channel::TextChannel { description, .. }
                | Channel::VoiceChannel { description, .. } => *description = None,
                Channel::SavedMessages { .. } | Channel::DirectMessage { .. } => {}
            },
            Self::Icon => match channel {
                Channel::Group { icon, .. }
                | Channel::TextChannel { icon, .. }
                | Channel::VoiceChannel { icon, .. } => *icon = None,
                Channel::SavedMessages { .. } | Channel::DirectMessage { .. } => {}
            },
        }
    }
}

/// An invite code
#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct ChannelInviteCode(String);

impl Display for ChannelInviteCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CreateChannelInviteResponse {
    code: ChannelInviteCode,
}

/// Message filter
#[derive(Debug, Default, Serialize)]
pub struct MessageFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<MessageId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<MessageId>,
    pub sort: MessageFilterSortDirection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nearby: Option<MessageId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_users: Option<bool>,
}

/// THe direction of a message filter
#[derive(Debug, Serialize)]
pub enum MessageFilterSortDirection {
    /// Tkae the latest messages first.
    Latest,
    /// Take the oldest messages first.
    Oldest,
}

impl Default for MessageFilterSortDirection {
    fn default() -> Self {
        Self::Latest
    }
}

/// Server channel type
#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ServerChannelType {
    Text,
    Voice,
}
