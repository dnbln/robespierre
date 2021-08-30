use std::fmt;

use robespierre_models::{
    channel::Channel,
    id::{ChannelId, UserId},
    user::User,
};

#[derive(Copy, Clone)]
pub enum Mention {
    Channel(ChannelId),
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

pub trait Mentionable {
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
