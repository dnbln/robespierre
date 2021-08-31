use serde::{Deserialize, Serialize};

use crate::id::AttachmentId;

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

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(tag = "type")]
pub enum AttachmentMetadata {
    File,
    Text,
    Audio,
    Image { width: usize, height: usize },
    Video { width: usize, height: usize },
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentTag {
    Attachments,
    Avatars,
    Backgrounds,
    Banners,
    Icons,
}

