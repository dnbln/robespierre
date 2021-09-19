use serde::{Deserialize, Serialize};

/*
Types
*/

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/January.ts#L1-L12

/// Embedded image
#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct EmbedImage {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub size: SizeType,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/January.ts#L14-L23

/// Embedded video
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct EmbedVideo {
    pub url: String,
    pub width: u32,
    pub height: u32,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/January.ts#L25-L35

/// Data about an embed of a special website, if it is the case
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(tag = "type")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub enum EmbedSpecial {
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

/// Size type
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum SizeType {
    Large,
    Preview,
}

// https://github.com/revoltchat/api/blob/097f40e37108cd3a1816b1c2cc69a137ae317069/types/January.ts#L37-L66

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[serde(tag = "type")]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub enum Embed {
    None,
    Website {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        special: Option<EmbedSpecial>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        image: Option<EmbedImage>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        video: Option<EmbedVideo>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        site_name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        icon_url: Option<String>,
        #[serde(rename = "colour", default, skip_serializing_if = "Option::is_none")]
        color: Option<String>,
    },
    Image(EmbedImage),
}
