use std::{ops::Deref, result::Result as StdResult};

use reqwest::{
    header::{HeaderMap, HeaderValue},
    multipart::{Form, Part},
    RequestBuilder,
};
use robespierre_models::{
    auth::Session,
    autumn::{AttachmentId, AttachmentTag},
    core::RevoltConfiguration,
    id::UserId,
};

/// Any error that can happen while requesting / decoding
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("decoding: {0}")]
    Decoding(#[from] serde_json::Error),
}

pub type Result<T = ()> = StdResult<T, HttpError>;

/// A value that can be used to authenticate on the REST API, either as a bot or as a non-bot user.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HttpAuthentication<'a> {
    BotToken { token: &'a str },
    UserSession { session_token: &'a str },
}
trait AuthExt: Sized {
    fn auth(self, auth: &HttpAuthentication) -> Self;
}

impl AuthExt for RequestBuilder {
    fn auth(self, auth: &HttpAuthentication) -> Self {
        match auth {
            HttpAuthentication::BotToken { token } => self.header("x-bot-token", *token),
            HttpAuthentication::UserSession { session_token } => {
                self.header("x-session-token", *session_token)
            }
        }
    }
}

impl AuthExt for HeaderMap {
    fn auth(mut self, auth: &HttpAuthentication) -> Self {
        match auth {
            HttpAuthentication::BotToken { token } => {
                self.insert("x-bot-token", token.parse().unwrap());
            }
            HttpAuthentication::UserSession { session_token } => {
                self.insert("x-session-token", session_token.parse().unwrap());
            }
        }

        self
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AuthType {
    Bot,
    UserSession,
}

macro_rules! ep {
    ($self:ident, $ep:literal $($args:tt)*) => {
        format!(concat!("{}", $ep), $self.api_root, $($args)*)
    };

    (api_root = $api_root:expr, $ep:literal $($args:tt)*) => {
        format!(concat!("{}", $ep), $api_root, $($args)*)
    };
}

macro_rules! autumn_tag_upload {
    ($self:expr, $tag:expr) => {
        format!("{}/{}", $self.revolt_config.features.autumn.url(), $tag)
    };
}

impl<'a> From<&'a Session> for HttpAuthentication<'a> {
    fn from(s: &'a Session) -> Self {
        HttpAuthentication::UserSession {
            session_token: &s.token.0,
        }
    }
}

/// An instance of a client to the REST API
pub struct Http {
    client: reqwest::Client,
    api_root: String,
    revolt_config: RevoltConfiguration,

    auth_type: AuthType,
}

pub mod core;

pub mod onboarding;

pub mod account;

pub mod session;

pub mod users_information;

pub mod direct_messaging;

pub mod relationships;

pub mod channel_information;

pub mod channel_invites;

pub mod channel_permissions;

pub mod messaging;

pub mod groups;

pub mod voice;

pub mod server_information;

pub mod server_members;

pub mod server_permissions;

pub mod bots;

pub mod invites;

pub mod sync;

pub mod web_push;

mod impl_prelude {
    pub use super::Http;
    pub use super::Result;
}

impl Http {
    /// Creates a new client from the authentication
    pub async fn new<'auth>(auth: impl Into<HttpAuthentication<'auth>>) -> Result<Self> {
        Self::new_with_url(auth, "https://api.revolt.chat").await
    }

    /// Creates a new client from the authentication and url.
    ///
    /// Use this if using a self hosted instance of revolt, otherwise use [`Self::new`].
    pub async fn new_with_url<'auth>(
        auth: impl Into<HttpAuthentication<'auth>>,
        api_root: &str,
    ) -> Result<Self> {
        let auth = auth.into();
        let mut default_headers = HeaderMap::new().auth(&auth);
        default_headers.insert(reqwest::header::ACCEPT, HeaderValue::from_static("*/*"));
        let client = reqwest::Client::builder()
            .default_headers(default_headers)
            .build()
            .unwrap();
        let revolt_config = Self::get_revolt_config(&client, api_root).await?;
        let auth_type = match auth {
            HttpAuthentication::BotToken { .. } => AuthType::Bot,
            HttpAuthentication::UserSession { .. } => AuthType::UserSession,
        };
        Ok(Self {
            client,
            api_root: api_root.to_string(),
            revolt_config,
            auth_type,
        })
    }

    fn client_user_session_auth_type(&self) -> &reqwest::Client {
        match self.auth_type {
            AuthType::Bot => panic!("Cannot use route when using a bot auth"),
            AuthType::UserSession => &self.client,
        }
    }

    /// Gets the websocket url
    pub fn get_ws_url(&self) -> &str {
        &self.revolt_config.ws
    }

    pub async fn get_self_id(&self) -> Result<UserId> {
        Ok(self.fetch_account().await?.id)
    }

    pub fn get_revolt_configuration(&self) -> &RevoltConfiguration {
        &self.revolt_config
    }

    /// Uploads a file to autumn, returning the [`AttachmentId`]
    pub async fn upload_autumn(
        &self,
        tag: AttachmentTag,
        name: String,
        bytes: Vec<u8>,
    ) -> Result<AttachmentId> {
        #[derive(serde::Deserialize)]
        struct AutumnUploadResponse {
            id: AttachmentId,
        }

        let part = Part::bytes(bytes).file_name(name.clone());
        let form = Form::new().part(name, part);
        let req = self
            .client
            .post(autumn_tag_upload!(self, tag))
            .multipart(form);
        let resp = req
            .send()
            .await?
            .error_for_status()?
            .json::<AutumnUploadResponse>()
            .await?;
        Ok(resp.id)
    }
}

pub trait HasHttp: Send + Sync {
    fn get_http(&self) -> &Http;
}

impl HasHttp for Http {
    fn get_http(&self) -> &Http {
        self
    }
}

impl<T: Deref<Target = Http> + Send + Sync> HasHttp for T {
    fn get_http(&self) -> &Http {
        &**self
    }
}
