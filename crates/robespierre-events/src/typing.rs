use std::{
    collections::{hash_map::Entry, HashMap},
    time::Duration,
};

use futures::Future;
use robespierre_models::id::ChannelId;
use tokio::time::Instant;

use crate::{ConnectionMessage, ConnectionMessanger};

/// A RAII-style typing session, which when dropped sends a StopTyping message to the [`ConnectionManager`].
#[derive(Clone, Debug)]
#[must_use = "Has to be dropped when the typing session ends"]
pub struct TypingSession {
    channel_id: ChannelId,
    messanger: ConnectionMessanger,
}

impl TypingSession {
    pub fn new(channel_id: ChannelId, messanger: ConnectionMessanger) -> Self {
        Self {
            channel_id,
            messanger,
        }
    }
}

impl Drop for TypingSession {
    fn drop(&mut self) {
        self.messanger.send(ConnectionMessage::StopTyping {
            channel: self.channel_id,
        })
    }
}

pub struct TypingSessionManager {
    sessions: HashMap<ChannelId, usize>,
    interval: tokio::time::Interval,
}

impl Default for TypingSessionManager {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            interval: tokio::time::interval(Duration::new(2, 500_000_000)),
        }
    }
}

impl TypingSessionManager {
    pub fn start_typing(&mut self, channel: ChannelId) {
        *self.sessions.entry(channel).or_insert(0) += 1;
    }

    pub fn stop_typing(&mut self, channel: ChannelId) -> bool {
        match self.sessions.entry(channel) {
            Entry::Occupied(mut entry) => {
                *entry.get_mut() -= 1;
                if *entry.get() == 0 {
                    entry.remove();
                    return true;
                }
            }
            Entry::Vacant(_) => {
                tracing::debug!("Trying to stop typing, but no session for the channel");
            }
        }

        false
    }

    pub fn current_sessions(&self) -> ChannelIdIter {
        ChannelIdIter(self.sessions.keys())
    }

    pub fn tick(&mut self) -> impl Future<Output = Instant> + '_ {
        self.interval.tick()
    }
}

pub struct ChannelIdIter<'a>(std::collections::hash_map::Keys<'a, ChannelId, usize>);

impl<'a> Iterator for ChannelIdIter<'a> {
    type Item = &'a ChannelId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
