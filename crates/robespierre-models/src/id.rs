use std::{
    convert::{TryFrom, TryInto},
    fmt::Display,
    str::FromStr,
};

use serde::{Deserialize, Serialize};

/// 26-bytes of numeric or uppercase letter characters
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(try_from = "String", into = "String")]
pub struct IdString([u8; 26]);

impl IdString {
    fn check(s: &str) -> Result<(), IdStringDeserializeError> {
        if s.len() != 26 {
            return Err(IdStringDeserializeError::IncorrectLength {
                expected: 26,
                len: s.len(),
            });
        }

        match s.find(|c: char| !('A'..='Z').contains(&c) && !('0'..='9').contains(&c)) {
            Some(pos) => {
                let c = s.chars().nth(pos).unwrap();

                return Err(IdStringDeserializeError::InvalidCharacter { c, pos });
            }

            None => {}
        }

        Ok(())
    }

    pub unsafe fn from_str_unchecked(s: &str) -> Self {
        Self(s.as_bytes().try_into().unwrap())
    }

    pub unsafe fn from_string_unchecked(s: String) -> Self {
        Self(s.as_bytes().try_into().unwrap())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum IdStringDeserializeError {
    #[error("invalid character {c} at position {pos}")]
    InvalidCharacter { pos: usize, c: char },
    #[error("incorrect length: is {len}, expected {expected}")]
    IncorrectLength { len: usize, expected: usize },
}

impl FromStr for IdString {
    type Err = IdStringDeserializeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::check(s)?;

        Ok(Self(s.as_bytes().try_into().unwrap()))
    }
}

impl TryFrom<String> for IdString {
    type Error = IdStringDeserializeError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::check(&s)?;

        Ok(Self(s.as_bytes().try_into().unwrap()))
    }
}

impl AsRef<str> for IdString {
    fn as_ref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl From<IdString> for String {
    fn from(id: IdString) -> Self {
        id.as_ref().to_string()
    }
}

impl Display for IdString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

macro_rules! id_impl {
    ($name:ident) => {
        impl From<IdString> for $name {
            fn from(id: IdString) -> Self {
                Self(id)
            }
        }

        impl From<$name> for IdString {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl FromStr for $name {
            type Err = <IdString as FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                s.parse().map(Self)
            }
        }

        impl TryFrom<String> for $name {
            type Error = <IdString as TryFrom<String>>::Error;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                IdString::try_from(value).map(Self)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.as_ref().to_string()
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct UserId(IdString);

id_impl! {UserId}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct ChannelId(IdString);

id_impl! {ChannelId}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct MessageId(IdString);

id_impl! {MessageId}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct ServerId(IdString);

id_impl! {ServerId}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct RoleId(IdString);

id_impl! {RoleId}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct MemberId {
    pub server: ServerId,
    pub user: UserId,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(try_from = "String", into = "String")]
pub struct AttachmentId([u8; 42]);

impl AttachmentId {
    fn check(s: &str) -> Result<(), IdStringDeserializeError> {
        if s.len() != 42 {
            return Err(IdStringDeserializeError::IncorrectLength {
                expected: 42,
                len: s.len(),
            });
        }

        match s.find(|c: char| {
            !('A'..='Z').contains(&c) && !('a'..='z').contains(&c) && !('0'..='9').contains(&c)
        }) {
            Some(pos) => {
                let c = s.chars().nth(pos).unwrap();

                return Err(IdStringDeserializeError::InvalidCharacter { c, pos });
            }

            None => {}
        }

        Ok(())
    }

    pub unsafe fn from_str_unchecked(s: &str) -> Self {
        Self(s.as_bytes().try_into().unwrap())
    }

    pub unsafe fn from_string_unchecked(s: String) -> Self {
        Self(s.as_bytes().try_into().unwrap())
    }
}

impl FromStr for AttachmentId {
    type Err = IdStringDeserializeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::check(s)?;

        Ok(Self(s.as_bytes().try_into().unwrap()))
    }
}

impl TryFrom<String> for AttachmentId {
    type Error = IdStringDeserializeError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::check(&s)?;

        Ok(Self(s.as_bytes().try_into().unwrap()))
    }
}

impl AsRef<str> for AttachmentId {
    fn as_ref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl From<AttachmentId> for String {
    fn from(id: AttachmentId) -> Self {
        id.as_ref().to_string()
    }
}

impl Display for AttachmentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}
