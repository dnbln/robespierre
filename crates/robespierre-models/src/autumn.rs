use std::str::FromStr;
use std::{convert::Infallible, fmt};

use serde::{Deserialize, Serialize};

/*
Newtypes
*/

/// Information about a file size
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct FileSize {
    bytes: usize,
}

impl FileSize {
    pub fn to_bytes(&self) -> usize {
        self.bytes
    }
}

/*
Types
*/

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Autumn.ts#L1-L6

/// Id type for attachments
///
/// Attachment ids are returned by `Autumn`.
// and can be from 1 up to 128 characters
// right now uploading a file gives a 42
// char id
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(try_from = "String", into = "String")]
pub struct AttachmentId([u8; 128], usize); // buffer + string slice length

impl FromStr for AttachmentId {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl<'a> From<&'a str> for AttachmentId {
    fn from(s: &'a str) -> Self {
        let len = s.len();
        assert!(len <= 128);
        let mut buf = [0; 128];
        buf[..len].copy_from_slice(s.as_bytes());

        Self(buf, len)
    }
}

impl From<String> for AttachmentId {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl AsRef<str> for AttachmentId {
    fn as_ref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0[..self.1]) }
    }
}

impl From<AttachmentId> for String {
    fn from(id: AttachmentId) -> Self {
        id.as_ref().to_string()
    }
}

impl fmt::Display for AttachmentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_ref().fmt(f)
    }
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Autumn.ts#L8-L14

/// Attachment metadata
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(tag = "type")]
pub enum AttachmentMetadata {
    File,
    Text,
    Audio,
    Image { width: usize, height: usize },
    Video { width: usize, height: usize },
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Autumn.ts#L16-L19

/// Attachment tag
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentTag {
    Attachments,
    Avatars,
    Backgrounds,
    Icons,
    Banners,
}

impl AttachmentTag {
    pub fn to_str(self) -> &'static str {
        match self {
            Self::Attachments => "attachments",
            Self::Avatars => "avatars",
            Self::Backgrounds => "backgrounds",
            Self::Icons => "icons",
            Self::Banners => "banners",
        }
    }
}

impl fmt::Display for AttachmentTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_str().fmt(f)
    }
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Autumn.ts#L21-L45

/// Attachment to a message, but can be any other media
/// like avatars, server icons, channel icons, banners
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Attachment {
    #[serde(rename = "_id")]
    pub id: AttachmentId,
    pub tag: AttachmentTag,
    pub size: FileSize,
    pub filename: String,
    pub metadata: AttachmentMetadata,
    pub content_type: String,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/Autumn.ts#L47-L70

/// File serving parameters
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct SizeOptions {
    /// Width of resized image
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// Height of resized image
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Width and height of resized image
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
    /// Maximum resized image side length
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_size: Option<u32>,
}
