use std::{
    cmp::Ordering,
    convert::{TryFrom, TryInto},
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

/*

Variable length id strings

*/

/// variable length(up to 26 bytes) of numeric or letter characters
#[derive(Serialize, Deserialize, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(try_from = "String", into = "String")]
pub struct VarLenIdString([u8; 26], usize);

impl VarLenIdString {
    /// Checks whether the string is a valid Id
    pub fn check(s: &str) -> Result<(), VarLenIdStringDeserializeError> {
        if s.len() > 26 {
            return Err(VarLenIdStringDeserializeError::IncorrectLength {
                expected_most: 26,
                len: s.len(),
            });
        }

        if let Some(pos) = s.find(|c: char| {
            !('A'..='Z').contains(&c) && !('a'..='z').contains(&c) && !('0'..='9').contains(&c)
        }) {
            let c = s.chars().nth(pos).unwrap();

            return Err(VarLenIdStringDeserializeError::InvalidCharacter { c, pos });
        }

        Ok(())
    }

    /// Creates a new [`IdString`] wihtout checking that
    /// the contents are valid id characters, and the whole string
    /// is of the right length.
    /// # Safety
    /// s should have passed [Self::check]
    pub unsafe fn from_str_unchecked(s: &str) -> Self {
        let len = s.len();
        let mut buf = [0; 26];
        buf[..len].copy_from_slice(s.as_bytes());

        Self(buf, len)
    }

    /// Creates a new [`IdString`] wihtout checking that
    /// the contents are valid id characters, and the whole string
    /// is of the right length.
    /// # Safety
    /// s should have passed [Self::check]
    pub unsafe fn from_string_unchecked(s: String) -> Self {
        Self::from_str_unchecked(&s)
    }
}

/// An error that can occur while parsing an IdString
#[derive(thiserror::Error, Debug)]
pub enum VarLenIdStringDeserializeError {
    #[error("invalid character '{c}' at position {pos}")]
    InvalidCharacter { pos: usize, c: char },
    #[error("incorrect length: is {len}, expected at most {expected_most}")]
    IncorrectLength { len: usize, expected_most: usize },
}

impl FromStr for VarLenIdString {
    type Err = VarLenIdStringDeserializeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::check(s)?;

        Ok(unsafe { Self::from_str_unchecked(s) })
    }
}

impl TryFrom<String> for VarLenIdString {
    type Error = VarLenIdStringDeserializeError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::check(&s)?;

        Ok(unsafe { Self::from_string_unchecked(s) })
    }
}

impl AsRef<str> for VarLenIdString {
    fn as_ref(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0[..self.1]) }
    }
}

impl From<VarLenIdString> for String {
    fn from(id: VarLenIdString) -> Self {
        id.as_ref().to_string()
    }
}

impl std::fmt::Debug for VarLenIdString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <str as Display>::fmt(self.as_ref(), f)
    }
}

impl Display for VarLenIdString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl PartialEq<String> for VarLenIdString {
    fn eq(&self, other: &String) -> bool {
        self.as_ref().eq(other)
    }
}

impl PartialEq<str> for VarLenIdString {
    fn eq(&self, other: &str) -> bool {
        self.as_ref().eq(other)
    }
}

impl<'a> PartialEq<&'a str> for VarLenIdString {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_ref().eq(*other)
    }
}

impl PartialOrd<String> for VarLenIdString {
    fn partial_cmp(&self, other: &String) -> Option<Ordering> {
        Some(self.as_ref().cmp(other))
    }
}

impl PartialOrd<str> for VarLenIdString {
    fn partial_cmp(&self, other: &str) -> Option<Ordering> {
        Some(self.as_ref().cmp(other))
    }
}

impl<'a> PartialOrd<&'a str> for VarLenIdString {
    fn partial_cmp(&self, other: &&'a str) -> Option<Ordering> {
        Some(self.as_ref().cmp(*other))
    }
}

macro_rules! id_impl {
    (@dt $name:ident, $base_ty:ty) => {
        impl $name {
            pub fn datetime(&self) -> chrono::DateTime<chrono::Utc> {
                self.0.datetime()
            }
        }

        id_impl!($name, $base_ty);
    };
    ($name:ident, $base_ty:ty) => {
        impl From<$base_ty> for $name {
            fn from(id: $base_ty) -> Self {
                Self(id)
            }
        }

        impl From<$name> for $base_ty {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl FromStr for $name {
            type Err = <$base_ty as FromStr>::Err;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                s.parse().map(Self)
            }
        }

        impl TryFrom<String> for $name {
            type Error = <$base_ty as TryFrom<String>>::Error;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                <$base_ty>::try_from(value).map(Self)
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

id_impl! {@dt UserId, IdString}

/// Id type for channels.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct ChannelId(IdString);

id_impl! {@dt ChannelId, IdString}

/// Id type for channels.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct CategoryId(VarLenIdString);

id_impl! {CategoryId, VarLenIdString}

/// Id type for messages.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct MessageId(IdString);

id_impl! {@dt MessageId, IdString}

/// Id type for servers.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct ServerId(IdString);

id_impl! {@dt ServerId, IdString}

/// Id type for roles.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct RoleId(IdString);

id_impl! {@dt RoleId, IdString}

/// Id type for sessions.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct SessionId(IdString);

id_impl! {@dt SessionId, IdString}

/// Id type for invites.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct InviteId(IdString);

id_impl! {@dt InviteId, IdString}

/// Id type for members.
///
/// Note: it is a pair of a [`ServerId`] and [`UserId`]
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct MemberId {
    pub server: ServerId,
    pub user: UserId,
}
