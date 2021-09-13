use std::result::Result as StdResult;

use reqwest::{
    header::{HeaderMap, HeaderValue},
    multipart::{Form, Part},
    RequestBuilder,
};
use robespierre_models::{
    auth::{Account, Session, SessionInfo},
    autumn::{Attachment, AttachmentId, AttachmentTag},
    bot::{Bot, BotField, PublicBot},
    channels::{
        Channel, ChannelField, ChannelInviteCode, ChannelPermissions, CreateChannelInviteResponse,
        DirectMessageChannel, Message, MessageFilter, ReplyData, ServerChannelType,
    },
    core::RevoltConfiguration,
    id::{ChannelId, InviteId, MessageId, RoleId, ServerId, SessionId, UserId},
    invites::RetrievedInvite,
    servers::{
        Ban, Member, MemberField, PartialMember, PartialServer, PermissionTuple, Server,
        ServerField,
    },
    users::{Profile, Relationship, RelationshipStatus, User, UserEditPatch},
};

use crate::utils::{PermissionsUpdateRequest, SendMessageRequest, SetPermissionsRequest};

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

/// An instance of a client to the REST API
pub struct Http {
    client: reqwest::Client,
    api_root: String,
    revolt_config: RevoltConfiguration,

    auth_type: AuthType,
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

impl Http {
    /// Creates a new client from the authentication
    pub async fn new<'auth>(auth: impl Into<HttpAuthentication<'auth>>) -> Result<Self> {
        Self::new_with_url(auth, "https://api.revolt.chat").await
    }

    /// Creates a new client from the authentication and url.
    ///
    /// Use this if using a self hosted instance of revolt, otherwise use [Self::new].
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

    async fn get_revolt_config(
        client: &reqwest::Client,
        root_url: &str,
    ) -> Result<RevoltConfiguration> {
        Ok(client
            .get(root_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    // onboarding
    pub async fn get_onboarding(&self) -> Result<OnboardingStatus> {
        Ok(self
            .client_user_session_auth_type()
            .get(ep!(self, "/onboard/hello"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn complete_onboarding(&self, username: &str) -> Result {
        #[derive(serde::Serialize)]
        struct CompleteOnboardingRequest<'a> {
            username: &'a str,
        }

        self.client_user_session_auth_type()
            .post(ep!(self, "/onboard/complete"))
            .json(&CompleteOnboardingRequest { username })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    // account
    pub async fn fetch_account(&self) -> Result<Account> {
        Ok(self
            .client_user_session_auth_type()
            .get(ep!(self, "/account"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn create_account(
        email: &str,
        password: &str,
        invite: Option<&str>,
        captcha: Option<&str>,
    ) -> Result {
        #[derive(serde::Serialize)]
        struct CreateAccountRequest<'a> {
            email: &'a str,
            password: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            invite: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            captcha: Option<&'a str>,
        }

        reqwest::Client::new()
            .post(ep!(api_root = "https://api.revolt.chat", "/account/create"))
            .json(&CreateAccountRequest {
                email,
                password,
                invite,
                captcha,
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn resend_verification(email: &str, captcha: Option<&str>) -> Result {
        #[derive(serde::Serialize)]
        struct ResendVerificationRequest<'a> {
            email: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            captcha: Option<&'a str>,
        }

        reqwest::Client::new()
            .post(ep!(
                api_root = "https://api.revolt.chat",
                "/account/reverify"
            ))
            .json(&ResendVerificationRequest { email, captcha })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn verify_email(code: &str) -> Result {
        reqwest::Client::new()
            .post(ep!(
                api_root = "https://api.revolt.chat",
                "/account/verify/{}" code
            ))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn send_password_reset(email: &str, captcha: Option<&str>) -> Result {
        #[derive(serde::Serialize)]
        struct SendPasswordResetRequest<'a> {
            email: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            captcha: Option<&'a str>,
        }

        reqwest::Client::new()
            .post(ep!(
                api_root = "https://api.revolt.chat",
                "/account/reset_password"
            ))
            .json(&SendPasswordResetRequest { email, captcha })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn password_reset(password: &str, token: &str) -> Result {
        #[derive(serde::Serialize)]
        struct PasswordResetRequest<'a> {
            password: &'a str,
            token: &'a str,
        }

        reqwest::Client::new()
            .patch(ep!(
                api_root = "https://api.revolt.chat",
                "/account/reset_password"
            ))
            .json(&PasswordResetRequest { password, token })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn change_password(&self, password: &str, current_password: &str) -> Result {
        #[derive(serde::Serialize)]
        struct ChangePasswordRequest<'a> {
            password: &'a str,
            current_password: &'a str,
        }

        self.client_user_session_auth_type()
            .post(ep!(self, "/account/change/password"))
            .json(&ChangePasswordRequest {
                password,
                current_password,
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn change_email(&self, email: &str, current_password: &str) -> Result {
        #[derive(serde::Serialize)]
        struct ChangeEmailRequest<'a> {
            email: &'a str,
            current_password: &'a str,
        }

        self.client_user_session_auth_type()
            .post(ep!(self, "/account/change/email"))
            .json(&ChangeEmailRequest {
                email,
                current_password,
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn login(
        email: &str,
        password: Option<&str>,
        challange: Option<&str>,
        friendly_name: Option<&str>,
        captcha: Option<&str>,
    ) -> Result<Session> {
        #[derive(serde::Serialize)]
        struct LoginRequest<'a> {
            email: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            password: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            challange: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            friendly_name: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            captcha: Option<&'a str>,
        }

        Ok(reqwest::Client::new()
            .post(ep!(api_root = "https://api.revolt.chat", "/session/login"))
            .json(&LoginRequest {
                email,
                password,
                challange,
                friendly_name,
                captcha,
            })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn logout(self) -> Result {
        self.client_user_session_auth_type()
            .delete(ep!(self, "/session/logout"))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn edit_session(&self, session: SessionId, friendly_name: &str) -> Result {
        #[derive(serde::Serialize)]
        struct EditSessionRequest<'a> {
            friendly_name: &'a str,
        }

        self.client_user_session_auth_type()
            .patch(ep!(self, "/session/{}" session))
            .json(&EditSessionRequest { friendly_name })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn delete_session(&self, session: SessionId) -> Result {
        self.client_user_session_auth_type()
            .delete(ep!(self, "/session/{}" session))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn fetch_sessions(&self) -> Result<Vec<SessionInfo>> {
        Ok(self
            .client_user_session_auth_type()
            .get(ep!(self, "/session/all"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn delete_all_sessions(&self, revoke_self: bool) -> Result {
        self.client_user_session_auth_type()
            .delete(ep!(self, "/session/all"))
            .query(&[("revoke_self", revoke_self)])
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Gets an user from the api
    pub async fn fetch_user(&self, user_id: UserId) -> Result<User> {
        Ok(self
            .client
            .get(ep!(self, "/users/{}" user_id))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Edits an user
    pub async fn edit_user(&self, patch: UserEditPatch) -> Result {
        self.client
            .patch(ep!(self, "/users/@me"))
            .json(&patch)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Edits an username
    pub async fn edit_username(&self, username: &str, password: &str) -> Result {
        #[derive(serde::Serialize)]
        struct EditUsernameRequest<'a> {
            username: &'a str,
            password: &'a str,
        }

        self.client_user_session_auth_type()
            .patch(ep!(self, "/users/@me/username"))
            .json(&EditUsernameRequest { username, password })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Gets information abot an user profile
    pub async fn fetch_user_profile(&self, user_id: UserId) -> Result<Profile> {
        Ok(self
            .client
            .get(ep!(self, "/users/{}/profile" user_id))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    // TODO: fetch default avatar

    pub async fn fetch_mutual_friends(&self, user_id: UserId) -> Result<Vec<UserId>> {
        Ok(self
            .client
            .get(ep!(self, "/users/{}/mutual" user_id))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Gets dm channels / groups.
    pub async fn fetch_dm_channels(&self) -> Result<Vec<Channel>> {
        Ok(self
            .client
            .get(ep!(self, "/users/dms"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Opens a dm with user
    pub async fn open_dm(&self, user_id: UserId) -> Result<DirectMessageChannel> {
        Ok(self
            .client
            .get(ep!(self, "/users/{}/dm" user_id))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Fetches relationships of the current user
    pub async fn fetch_relationships(&self) -> Result<Vec<Relationship>> {
        Ok(self
            .client_user_session_auth_type()
            .get(ep!(self, "/users/relationships"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Fetches your relationship with the given user
    pub async fn fetch_relationship(&self, user_id: UserId) -> Result<SingleRelationshipResponse> {
        Ok(self
            .client_user_session_auth_type()
            .get(ep!(self, "/users/{}/relationship" user_id))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Sends or accepts a friend request to / from the user with given username
    pub async fn send_friend_request(&self, username: &str) -> Result<SingleRelationshipResponse> {
        Ok(self
            .client_user_session_auth_type()
            .put(ep!(self, "/users/{}/friend" username))
            .send()
            .await?
            .error_for_status()?
            .json::<SingleRelationshipResponse>()
            .await?)
    }

    /// Denies a friend request
    pub async fn deny_friend_request(&self, username: &str) -> Result<SingleRelationshipResponse> {
        Ok(self
            .client_user_session_auth_type()
            .delete(ep!(self, "/users/{}/friend" username))
            .send()
            .await?
            .error_for_status()?
            .json::<SingleRelationshipResponse>()
            .await?)
    }

    pub async fn remove_friend(&self, username: &str) -> Result<SingleRelationshipResponse> {
        self.deny_friend_request(username).await
    }

    /// Blocks an user
    pub async fn block_user(&self, user_id: UserId) -> Result<SingleRelationshipResponse> {
        Ok(self
            .client_user_session_auth_type()
            .put(ep!(self, "/users/{}/block" user_id))
            .send()
            .await?
            .error_for_status()?
            .json::<SingleRelationshipResponse>()
            .await?)
    }

    /// Unblocks an user
    pub async fn unblock(&self, user_id: UserId) -> Result<SingleRelationshipResponse> {
        Ok(self
            .client_user_session_auth_type()
            .delete(ep!(self, "/users/{}/block" user_id))
            .send()
            .await?
            .error_for_status()?
            .json::<SingleRelationshipResponse>()
            .await?)
    }

    /// Gets the channel given the id
    pub async fn fetch_channel(&self, channel_id: ChannelId) -> Result<Channel> {
        Ok(self
            .client
            .get(ep!(self, "/channels/{}" channel_id))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Edits the channel given by id
    pub async fn edit_channel(
        &self,
        channel_id: ChannelId,
        name: Option<String>,
        description: Option<String>,
        icon: Option<AttachmentId>,
        nsfw: Option<bool>,
        remove: Option<ChannelField>,
    ) -> Result {
        #[derive(serde::Serialize)]
        struct PatchChannelRequest {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            icon: Option<AttachmentId>,
            #[serde(skip_serializing_if = "Option::is_none")]
            nsfw: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            remove: Option<ChannelField>,
        }

        self.client
            .patch(ep!(self, "/channels/{}" channel_id))
            .json(&PatchChannelRequest {
                name,
                description,
                icon,
                nsfw,
                remove,
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Closes a channel / leaves group
    pub async fn close_channel(&self, channel_id: ChannelId) -> Result {
        self.client
            .delete(ep!(self, "/channels/{}" channel_id))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Creates an invite
    pub async fn create_invite(
        &self,
        channel_id: ChannelId,
    ) -> Result<CreateChannelInviteResponse> {
        Ok(self
            .client
            .post(ep!(self, "/channels/{}/invites" channel_id))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Sets role permissions
    pub async fn set_channel_role_permissions(
        &self,
        channel_id: ChannelId,
        role_id: RoleId,
        permissions: ChannelPermissions,
    ) -> Result {
        self.client
            .put(ep!(self, "/channels/{}/permissions/{}" channel_id, role_id))
            .json(&PermissionsUpdateRequest { permissions })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Sets default permissions
    pub async fn set_channel_default_role_permissions(
        &self,
        channel_id: ChannelId,
        permissions: ChannelPermissions,
    ) -> Result {
        self.client
            .put(ep!(self, "/channels/{}/permissions/default" channel_id))
            .json(&PermissionsUpdateRequest { permissions })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Sends a message
    pub async fn send_message(
        &self,
        channel_id: ChannelId,
        content: impl AsRef<str>,
        nonce: impl AsRef<str>,
        attachments: Vec<AttachmentId>,
        replies: Vec<ReplyData>,
    ) -> Result<Message> {
        Ok(self
            .client
            .post(ep!(self, "/channels/{}/messages" channel_id))
            .json(&SendMessageRequest {
                content: content.as_ref(),
                nonce: nonce.as_ref(),
                attachments,
                replies,
            })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Fetches messages
    pub async fn fetch_messages(
        &self,
        channel: ChannelId,
        filter: MessageFilter,
    ) -> Result<FetchMessagesResult> {
        let v = self
            .client
            .get(ep!(self, "/channels/{}/messages" channel))
            .json(&filter)
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;

        if matches!(&filter.include_users, Some(true)) {
            Ok(serde_json::from_value(v)?)
        } else {
            Ok(FetchMessagesResult {
                messages: serde_json::from_value(v)?,
                users: vec![],
                members: vec![],
            })
        }
    }

    /// Fetches a single message
    pub async fn fetch_message(&self, channel: ChannelId, message: MessageId) -> Result<Message> {
        Ok(self
            .client
            .get(ep!(self, "/channels/{}/messages/{}" channel, message))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Edits a message to contain content `content`
    pub async fn edit_message(
        &self,
        channel: ChannelId,
        message: MessageId,
        content: &str,
    ) -> Result {
        #[derive(serde::Serialize)]
        struct MessagePatch<'a> {
            content: &'a str,
        }
        self.client
            .patch(ep!(self, "/channels/{}/messages/{}" channel, message))
            .json(&MessagePatch { content })
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Deletes a message
    pub async fn delete_message(&self, channel: ChannelId, message: MessageId) -> Result {
        self.client
            .delete(ep!(self, "/channels/{}/messages/{}" channel, message))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn poll_message_changes(
        &self,
        channel: ChannelId,
        ids: &[MessageId],
    ) -> Result<PollMessageChangesResponse> {
        #[derive(serde::Serialize)]
        struct PollMessageChanges<'a> {
            ids: &'a [MessageId],
        }

        Ok(self
            .client
            .post(ep!(self, "/channels/{}/messages/stale" channel))
            .json(&PollMessageChanges { ids })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    // TODO: search for messages

    pub async fn acknowledge_message(&self, channel: ChannelId, message: MessageId) -> Result {
        self.client_user_session_auth_type()
            .put(ep!(self, "/channels/{}/ack/{}" channel, message))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Creates a group
    pub async fn create_group(
        &self,
        name: String,
        description: Option<String>,
        nonce: String,
        users: Option<&[UserId]>,
        nsfw: Option<bool>,
    ) -> Result<Channel> {
        #[derive(serde::Serialize)]
        struct CreateGroupRequest<'a> {
            name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            nonce: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            users: Option<&'a [UserId]>,
            #[serde(skip_serializing_if = "Option::is_none")]
            nsfw: Option<bool>,
        }
        Ok(self
            .client
            .post(ep!(self, "/channels/create"))
            .json(&CreateGroupRequest {
                name,
                description,
                nonce,
                users,
                nsfw,
            })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Fetches group members
    pub async fn fetch_group_members(&self, group: ChannelId) -> Result<Vec<User>> {
        Ok(self
            .client
            .get(ep!(self, "/channels/{}/members" group))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    // TODO: add member to group
    // TODO: remove member from group

    /// Fetches a server
    pub async fn fetch_server(&self, server: ServerId) -> Result<Server> {
        Ok(self
            .client
            .get(ep!(self, "/servers/{}" server))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Edits a server
    pub async fn edit_server(
        &self,
        server_id: ServerId,
        server: PartialServer,
        remove: ServerField,
    ) -> Result {
        #[derive(serde::Serialize)]
        struct ServerPatchRequest {
            #[serde(flatten)]
            server: PartialServer,
            remove: ServerField,
        }

        self.client
            .patch(ep!(self, "/servers/{}" server_id))
            .json(&ServerPatchRequest { server, remove })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Deletes a server
    pub async fn delete_server(&self, server_id: ServerId) -> Result {
        self.client
            .delete(ep!(self, "/servers/{}" server_id))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Creates a server
    pub async fn create_server(
        &self,
        name: String,
        description: Option<String>,
        nsfw: Option<bool>,
        nonce: String,
    ) -> Result<Server> {
        #[derive(serde::Serialize)]
        struct CreateServerRequest {
            name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            nsfw: Option<bool>,
            nonce: String,
        }
        Ok(self
            .client_user_session_auth_type()
            .post(ep!(self, "/servers/create"))
            .json(&CreateServerRequest {
                name,
                description,
                nsfw,
                nonce,
            })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Creates a channel
    pub async fn create_channel(
        &self,
        server: ServerId,
        kind: ServerChannelType,
        name: String,
        description: Option<String>,
        nonce: String,
    ) -> Result {
        #[derive(serde::Serialize)]
        struct CreateServerChannelRequest {
            #[serde(rename = "type")]
            kind: ServerChannelType,
            name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            nonce: String,
        }

        self.client
            .post(ep!(self, "/servers/{}/channels" server))
            .json(&CreateServerChannelRequest {
                kind,
                name,
                description,
                nonce,
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Fetches invites in server
    pub async fn fetch_invites(&self, server: ServerId) -> Result<Vec<FetchInviteResult>> {
        Ok(self
            .client
            .get(ep!(self, "/servers/{}/invites" server))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Marks server as read
    pub async fn mark_server_as_read(&self, server: ServerId) -> Result {
        self.client_user_session_auth_type()
            .put(ep!(self, "/servers/{}/ack" server))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Fetches member in server
    pub async fn fetch_member(&self, server_id: ServerId, user_id: UserId) -> Result<Member> {
        Ok(self
            .client
            .get(ep!(self, "/servers/{}/members/{}" server_id, user_id))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Edits member in server
    pub async fn edit_member(
        &self,
        server_id: ServerId,
        user_id: UserId,
        member: PartialMember,
        remove: MemberField,
    ) -> Result {
        #[derive(serde::Serialize)]
        struct PatchMemberRequest {
            #[serde(flatten)]
            member: PartialMember,
            remove: MemberField,
        }
        self.client
            .patch(ep!(self, "/servers/{}/members/{}" server_id, user_id))
            .json(&PatchMemberRequest { member, remove })
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    /// Kicks member out of server
    pub async fn kick_member(&self, server_id: ServerId, user_id: UserId) -> Result {
        self.client
            .delete(ep!(self, "/servers/{}/members/{}" server_id, user_id))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Fetches all members in a server
    pub async fn fetch_all_members(&self, server_id: ServerId) -> Result<FetchMembersResult> {
        Ok(self
            .client
            .get(ep!(self, "/servers/{}/members" server_id))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Bans an user from the server
    pub async fn ban_user(
        &self,
        server_id: ServerId,
        user_id: UserId,
        reason: Option<&str>,
    ) -> Result {
        #[derive(serde::Serialize)]
        struct BanRequest<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            reason: Option<&'a str>,
        }
        self.client
            .put(ep!(self, "/servers/{}/bans/{}" server_id, user_id))
            .json(&BanRequest { reason })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Unbans an user from the server
    pub async fn unban_user(&self, server_id: ServerId, user_id: UserId) -> Result {
        self.client
            .delete(ep!(self, "/servers/{}/bans/{}" server_id, user_id))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Fetches all the users who are banned and the reasons associated with their bans if available
    pub async fn fetch_bans(&self, server_id: ServerId) -> Result<FetchBansResult> {
        Ok(self
            .client
            .get(ep!(self, "/servers/{}/bans" server_id))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn set_role_permissions(
        &self,
        server: ServerId,
        role: RoleId,
        permissions: PermissionTuple,
    ) -> Result {
        let (sp, cp) = permissions;

        self.client
            .put(ep!(self, "/servers/{}/permissions/{}" server, role))
            .json(&SetPermissionsRequest {
                server: sp,
                channel: cp,
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn set_default_role_permissions(
        &self,
        server: ServerId,
        permissions: PermissionTuple,
    ) -> Result {
        let (sp, cp) = permissions;

        self.client
            .put(ep!(self, "/servers/{}/permissions/default" server))
            .json(&SetPermissionsRequest {
                server: sp,
                channel: cp,
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn create_role(&self, server: ServerId, name: &str) -> Result<RoleIdAndPermissions> {
        #[derive(serde::Serialize)]
        struct CreateRoleRequest<'a> {
            name: &'a str,
        }

        Ok(self
            .client
            .post(ep!(self, "/servers/{}/roles" server))
            .json(&CreateRoleRequest { name })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    // TODO: edit role

    pub async fn delete_role(&self, server: ServerId, role: RoleId) -> Result {
        self.client
            .delete(ep!(self, "/servers/{}/roles/{}" server, role))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn create_bot(&self, name: &str) -> Result<Bot> {
        #[derive(serde::Serialize)]
        struct CreateBotRequest<'a> {
            name: &'a str,
        }

        Ok(self
            .client_user_session_auth_type()
            .post(ep!(self, "/bots/create"))
            .json(&CreateBotRequest { name })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn fetch_owned_bots(&self) -> Result<FetchOwnedBotsResponse> {
        Ok(self
            .client_user_session_auth_type()
            .get(ep!(self, "/bots/@me"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn fetch_bot(&self, bot: UserId) -> Result<FetchBotResponse> {
        Ok(self
            .client_user_session_auth_type()
            .get(ep!(self, "/bots/{}" bot))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn edit_bot(
        &self,
        bot: UserId,
        name: Option<&str>,
        public: Option<bool>,
        interactions_url: Option<&str>,
        remove: Option<BotField>,
    ) -> Result {
        #[derive(serde::Serialize)]
        struct EditBotRequest<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            public: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            interactions_url: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            remove: Option<BotField>,
        }

        self.client_user_session_auth_type()
            .patch(ep!(self, "/bots/{}" bot))
            .json(&EditBotRequest {
                name,
                public,
                interactions_url,
                remove,
            })
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn delete_bot(&self, bot: UserId) -> Result {
        self.client_user_session_auth_type()
            .delete(ep!(self, "/bots/{}" bot))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn fetch_public_bot(&self, bot: UserId) -> Result<PublicBot> {
        Ok(self
            .client_user_session_auth_type()
            .get(ep!(self, "/bots/{}/invite" bot))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn invite_bot(&self, bot: UserId, target: InviteBotTarget) -> Result {
        self.client_user_session_auth_type()
            .post(ep!(self, "/bots/{}/invite" bot))
            .json(&target)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn fetch_invite(&self, invite: &str) -> Result<RetrievedInvite> {
        Ok(self
            .client
            .get(ep!(self, "/invites/{}" invite))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn join_invite(&self, invite: &str) -> Result<JoinInviteResponse> {
        Ok(self
            .client_user_session_auth_type()
            .post(ep!(self, "/invites/{}" invite))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn delete_invite(&self, invite: &str) -> Result {
        self.client
            .delete(ep!(self, "/invites/{}" invite))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    // TODO sync

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

/// Result when fetching multiple messages
#[derive(serde::Deserialize)]
pub struct FetchMessagesResult {
    pub messages: Vec<Message>,
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(default)]
    pub members: Vec<Member>,
}

/// Result when fetching members
#[derive(serde::Deserialize)]
pub struct FetchMembersResult {
    pub users: Vec<User>,
    pub members: Vec<Member>,
}

/// Result when fetching bans
#[derive(serde::Deserialize)]
pub struct FetchBansResult {
    pub users: Vec<FetchBansUser>,
    pub bans: Vec<Ban>,
}

#[derive(serde::Deserialize)]
pub struct FetchBansUser {
    #[serde(rename = "_id")]
    pub id: UserId,
    pub username: String,
    #[serde(default)]
    pub avatar: Option<Attachment>,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum FetchInviteResult {
    Server {
        #[serde(rename = "_id")]
        id: ChannelInviteCode,
        server: ServerId,
        creator: UserId,
        channel: ChannelId,
    },
}

#[derive(serde::Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct SingleRelationshipResponse {
    pub status: RelationshipStatus,
}

#[derive(serde::Deserialize)]
pub struct OnboardingStatus {
    pub onboarding: bool,
}

#[derive(serde::Deserialize)]
pub struct PollMessageChangesResponse {
    pub changed: Vec<Message>,
    pub deleted: Vec<MessageId>,
}

#[derive(serde::Deserialize)]
pub struct RoleIdAndPermissions {
    pub id: RoleId,
    pub permissions: PermissionTuple,
}

#[derive(serde::Deserialize)]
pub struct FetchOwnedBotsResponse {
    pub bots: Vec<Bot>,
    pub users: Vec<User>,
}

#[derive(serde::Deserialize)]
pub struct FetchBotResponse {
    pub bot: Bot,
    pub user: User,
}

pub enum InviteBotTarget {
    Server(ServerId),
    Group(ChannelId),
}

impl serde::Serialize for InviteBotTarget {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            InviteBotTarget::Server(server) => {
                #[derive(serde::Serialize)]
                struct TargetServer {
                    server: ServerId,
                }

                TargetServer { server: *server }.serialize(serializer)
            }
            InviteBotTarget::Group(group) => {
                #[derive(serde::Serialize)]
                struct TargetGroup {
                    group: ChannelId,
                }

                TargetGroup { group: *group }.serialize(serializer)
            }
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(tag = "type")]
pub enum JoinInviteResponse {
    Server(JoinServerInviteResponse),
}

#[derive(serde::Deserialize)]
pub struct JoinServerInviteResponse {
    pub channel: Channel,
    pub server: Server,
}

mod utils {
    use robespierre_models::{
        autumn::AttachmentId,
        channels::{ChannelPermissions, ReplyData},
        servers::ServerPermissions,
    };

    use serde::Serialize;

    #[derive(Serialize)]
    pub struct PermissionsUpdateRequest {
        pub permissions: ChannelPermissions,
    }

    #[derive(Serialize)]
    pub struct SendMessageRequest<'a> {
        pub content: &'a str,
        pub nonce: &'a str,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub attachments: Vec<AttachmentId>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub replies: Vec<ReplyData>,
    }

    #[derive(Serialize)]
    pub struct SetPermissionsRequest {
        pub server: ServerPermissions,
        pub channel: ChannelPermissions,
    }
}
