use std::{convert::TryFrom, fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

/// 26-bytes of numeric or uppercase letter characters
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(try_from = "String", into = "String")]
pub struct IdString(String);

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
        Self::from_string_unchecked(s.to_string())
    }

    pub unsafe fn from_string_unchecked(s: String) -> Self {
        Self(s)
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

        Ok(Self(s.to_string()))
    }
}

impl TryFrom<String> for IdString {
    type Error = IdStringDeserializeError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::check(&s)?;

        Ok(Self(s))
    }
}

impl From<IdString> for String {
    fn from(id: IdString) -> Self {
        id.0
    }
}

impl Display for IdString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
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

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                String::from(id.0)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct UserId(IdString);

id_impl! {UserId}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct AttachmentId(IdString);

id_impl! {AttachmentId}


#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct ChannelId(IdString);

id_impl! {ChannelId}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct MessageId(IdString);

id_impl! {MessageId}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct ServerId(IdString);

id_impl! {ServerId}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct RoleId(IdString);

id_impl! {RoleId}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct MemberId {
    pub server: ServerId,
    pub user: UserId,
}
