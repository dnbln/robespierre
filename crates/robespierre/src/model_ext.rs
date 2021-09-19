use robespierre_events::typing::TypingSession;
use robespierre_models::id::ChannelId;

use crate::Context;

pub trait AsRefContext: AsRef<Context> + Send + Sync {}
impl<T> AsRefContext for T where T: AsRef<Context> + Send + Sync {}

pub trait ChannelIdExt2 {
    #[cfg(feature = "events")]
    fn start_typing(&self, ctx: &impl AsRefContext) -> TypingSession;
}

impl ChannelIdExt2 for ChannelId {
    #[cfg(feature = "events")]
    fn start_typing(&self, ctx: &impl AsRefContext) -> TypingSession {
        ctx.as_ref().start_typing(*self)
    }
}