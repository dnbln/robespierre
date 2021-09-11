use std::{
    cmp::Ordering,
    convert::{Infallible, TryFrom, TryInto},
    fmt::Display,
    str::FromStr,
};

use serde::{Deserialize, Serialize};

/// 26-bytes of numeric or uppercase letter characters
#[derive(Serialize, Deserialize, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(try_from = "String", into = "String")]
pub struct IdString([u8; 26]);

impl IdString {
    /// Checks whether the string is a valid Id
    pub fn check(s: &str) -> Result<(), IdStringDeserializeError> {
        if s.len() != 26 {
            return Err(IdStringDeserializeError::IncorrectLength {
                expected: 26,
                len: s.len(),
            });
        }

        if let Some(pos) = s.find(|c: char| !('A'..='Z').contains(&c) && !('0'..='9').contains(&c))
        {
            let c = s.chars().nth(pos).unwrap();

            return Err(IdStringDeserializeError::InvalidCharacter { c, pos });
        }

        Ok(())
    }

    /// Creates a new [`IdString`] wihtout checking that
    /// the contents are valid id characters, and the whole string
    /// is of the right length.
    /// # Safety
    /// s should have passed [Self::check]
    pub unsafe fn from_str_unchecked(s: &str) -> Self {
        Self(s.as_bytes().try_into().unwrap())
    }

    /// Creates a new [`IdString`] wihtout checking that
    /// the contents are valid id characters, and the whole string
    /// is of the right length.
    /// # Safety
    /// s should have passed [Self::check]
    pub unsafe fn from_string_unchecked(s: String) -> Self {
        Self(s.as_bytes().try_into().unwrap())
    }

    /// Gets the time the object identified by this id was created at.
    pub fn datetime(&self) -> chrono::DateTime<chrono::Utc> {
        let ulid = self.as_ref().parse::<rusty_ulid::Ulid>().unwrap();

        ulid.datetime()
    }
}

/// An error that can occur while parsing an IdString
#[derive(thiserror::Error, Debug)]
pub enum IdStringDeserializeError {
    #[error("invalid character '{c}' at position {pos}")]
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

impl std::fmt::Debug for IdString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <str as Display>::fmt(self.as_ref(), f)
    }
}

impl Display for IdString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl PartialEq<String> for IdString {
    fn eq(&self, other: &String) -> bool {
        self.as_ref().eq(other)
    }
}

impl PartialEq<str> for IdString {
    fn eq(&self, other: &str) -> bool {
        self.as_ref().eq(other)
    }
}

impl<'a> PartialEq<&'a str> for IdString {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref().eq(*other)
    }
}

impl PartialOrd<String> for IdString {
    fn partial_cmp(&self, other: &String) -> Option<Ordering> {
        Some(self.as_ref().cmp(other))
    }
}

impl PartialOrd<str> for IdString {
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        Some(self.as_ref().cmp(other))
    }
}

impl<'a> PartialOrd<&'a str> for IdString {
    fn partial_cmp(&self, other: &&'a str) -> Option<Ordering> {
        Some(self.as_ref().cmp(*other))
    }
}

macro_rules! id_impl {
    ($name:ident) => {
        impl $name {
            pub fn datetime(&self) -> chrono::DateTime<chrono::Utc> {
                self.0.datetime()
            }
        }

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

        impl PartialEq<String> for $name {
            fn eq(&self, other: &String) -> bool {
                self.as_ref().eq(other)
            }
        }

        impl PartialEq<str> for $name {
            fn eq(&self, other: &str) -> bool {
                self.as_ref().eq(other)
            }
        }

        impl<'a> PartialEq<&'a str> for $name {
            fn eq(&self, other: &&'a str) -> bool {
                self.as_ref().eq(*other)
            }
        }

        impl PartialOrd<String> for $name {
            fn partial_cmp(&self, other: &String) -> Option<Ordering> {
                Some(self.as_ref().cmp(other))
            }
        }

        impl PartialOrd<str> for $name {
            fn partial_cmp(&self, other: &str) -> Option<Ordering> {
                Some(self.as_ref().cmp(other))
            }
        }

        impl<'a> PartialOrd<&'a str> for $name {
            fn partial_cmp(&self, other: &&'a str) -> Option<Ordering> {
                Some(self.as_ref().cmp(*other))
            }
        }
    };
}

/// Id type for users.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct UserId(IdString);

id_impl! {UserId}

/// Id type for channels.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct ChannelId(IdString);

id_impl! {ChannelId}

/// Id type for messages.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct MessageId(IdString);

id_impl! {MessageId}

/// Id type for servers.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct ServerId(IdString);

id_impl! {ServerId}

/// Id type for roles.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct RoleId(IdString);

id_impl! {RoleId}

/// Id type for members.
///
/// Note: it is a pair of a [`ServerId`] and [`UserId`]
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct MemberId {
    pub server: ServerId,
    pub user: UserId,
}

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

impl Display for AttachmentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}
