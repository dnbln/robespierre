//! Contains [`Mention`], a helper to format mentions from ids.

use std::fmt;

use robespierre_models::{
    channel::Channel,
    id::{ChannelId, UserId},
    user::User,
};

/// A helper which when formatted using [`std::fmt::Display`] creates a mention.

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Mention {
    /// A channel mention (`<#id>`)
    Channel(ChannelId),
    /// An user mention (`<@id>`)
    User(UserId),
}

impl From<UserId> for Mention {
    fn from(id: UserId) -> Self {
        Self::User(id)
    }
}

impl<'a> From<&'a User> for Mention {
    fn from(user: &'a User) -> Self {
        Self::User(user.id)
    }
}

impl From<User> for Mention {
    fn from(user: User) -> Self {
        Self::User(user.id)
    }
}

impl From<ChannelId> for Mention {
    fn from(id: ChannelId) -> Self {
        Self::Channel(id)
    }
}

impl<'a> From<&'a Channel> for Mention {
    fn from(channel: &'a Channel) -> Self {
        Self::Channel(channel.id())
    }
}

impl From<Channel> for Mention {
    fn from(channel: Channel) -> Self {
        Self::Channel(channel.id())
    }
}

impl fmt::Display for Mention {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mention::User(id) => write!(f, "<@{}>", id),
            Mention::Channel(id) => write!(f, "<#{}>", id),
        }
    }
}

/// Helper trait to create a [`Mention`] from any mentionable object.
/// ```rust
/// use robespierre::model::mention::{Mention, Mentionable};
/// use robespierre_models::id::{UserId, ChannelId};
/// # let s = "A".repeat(26);
/// let user_id: UserId = s.parse().unwrap();
/// let channel_id: ChannelId = s.parse().unwrap();
/// assert_eq!(user_id.mention(), Mention::User(user_id));
/// assert_eq!(channel_id.mention(), Mention::Channel(channel_id));
/// ```
pub trait Mentionable {
    /// Creates a mention
    fn mention(&self) -> Mention;
}

impl Mentionable for User {
    fn mention(&self) -> Mention {
        Mention::from(self.id)
    }
}

impl Mentionable for UserId {
    fn mention(&self) -> Mention {
        Mention::from(*self)
    }
}

impl Mentionable for Channel {
    fn mention(&self) -> Mention {
        Mention::from(self.id())
    }
}

impl Mentionable for ChannelId {
    fn mention(&self) -> Mention {
        Mention::from(*self)
    }
}

#[cfg(test)]
mod tests {
    use robespierre_models::id::{ChannelId, UserId};

    use super::Mention;

    #[test]
    fn mention_user_display() {
        let id = "A".repeat(26);
        
        let user_id: UserId = id.parse().unwrap();
        let mention = Mention::User(user_id);

        assert_eq!("<@AAAAAAAAAAAAAAAAAAAAAAAAAA>", &format!("{}", mention));
    }

    #[test]
    fn mention_channel_display() {
        let id = "A".repeat(26);
        
        let channel_id: ChannelId = id.parse().unwrap();
        let mention = Mention::Channel(channel_id);

        assert_eq!("<#AAAAAAAAAAAAAAAAAAAAAAAAAA>", &format!("{}", mention));
    }
}
