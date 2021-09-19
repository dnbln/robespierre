pub extern crate robespierre_http;
pub extern crate robespierre_models;

#[cfg(feature = "cache")]
pub extern crate robespierre_cache;

#[cfg(feature = "events")]
pub extern crate robespierre_events;

use robespierre_cache::{Cache, HasCache};
use robespierre_events::EventsError;
use robespierre_http::{HasHttp, Http, HttpAuthentication, HttpError};

pub mod model;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http error")]
    Http(#[from] HttpError),
    #[cfg(feature = "events")]
    #[error("events error")]
    Events(#[from] EventsError),
}

pub type Result<T = ()> = std::result::Result<T, Error>;

#[cfg(feature = "cache")]
pub trait CacheHttp: HasCache {
    fn http(&self) -> &Http;
    fn cache(&self) -> Option<&Cache>;
}

#[cfg(feature = "cache")]
impl<T: HasCache + HasHttp> CacheHttp for T {
    fn http(&self) -> &Http {
        self.get_http()
    }

    fn cache(&self) -> Option<&Cache> {
        self.get_cache()
    }
}

#[cfg(not(feature = "cache"))]
pub trait CacheHttp {
    fn http(&self) -> &Http;
}

#[cfg(not(feature = "cache"))]
impl<T: HasHttp> CacheHttp for T {
    fn http(&self) -> &Http {
        self.get_http()
    }
}

#[derive(Clone)]
pub enum Authentication {
    Bot { token: String },
    User { session_token: String },
}

impl Authentication {
    pub fn bot(token: impl Into<String>) -> Self {
        Self::Bot {
            token: token.into(),
        }
    }

    pub fn user(session_token: impl Into<String>) -> Self {
        Self::User {
            session_token: session_token.into(),
        }
    }
}

#[cfg(feature = "events")]
impl<'a> From<&'a Authentication> for robespierre_events::Authentication<'a> {
    fn from(auth: &'a Authentication) -> Self {
        match auth {
            Authentication::Bot { token } => Self::Bot {
                token: token.as_str(),
            },
            Authentication::User { session_token } => Self::User {
                session_token: session_token.as_str(),
            },
        }
    }
}

impl<'a> From<&'a Authentication> for HttpAuthentication<'a> {
    fn from(auth: &'a Authentication) -> Self {
        match auth {
            Authentication::Bot { token } => Self::BotToken {
                token: token.as_str(),
            },
            Authentication::User { session_token } => Self::UserSession {
                session_token: session_token.as_str(),
            },
        }
    }
}
