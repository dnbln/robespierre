use std::result::Result as StdResult;

use reqwest::{
    header::{HeaderMap, HeaderValue},
    multipart::{Form, Part},
    RequestBuilder,
};
use robespierre_models::{
    autumn::{Attachment, AttachmentId, AttachmentTag},
    channels::{
        Channel, ChannelField, ChannelInviteCode, ChannelPermissions, CreateChannelInviteResponse,
        Message, MessageFilter, ReplyData, ServerChannelType,
    },
    core::RevoltConfiguration,
    id::{ChannelId, MessageId, RoleId, ServerId, UserId},
    servers::{Ban, Member, MemberField, PartialMember, PartialServer, Server, ServerField},
    users::{Profile, Relationship, RelationshipStatus, User, UserEditPatch},
};

use crate::utils::{PermissionsUpdateRequest, SendMessageRequest};

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
    BotToken {
        token: &'a str,
    },
    UserSession {
        user_id: UserId,
        session_token: &'a str,
    },
}
trait AuthExt: Sized {
    fn auth(self, auth: &HttpAuthentication) -> Self;
}

impl AuthExt for RequestBuilder {
    fn auth(self, auth: &HttpAuthentication) -> Self {
        match auth {
            HttpAuthentication::BotToken { token } => self.header("x-bot-token", *token),
            HttpAuthentication::UserSession {
                user_id,
                session_token,
            } => self
                .header("x-session-token", *session_token)
                .header("x-user-id", format!("{}", user_id)),
        }
    }
}

impl AuthExt for HeaderMap {
    fn auth(mut self, auth: &HttpAuthentication) -> Self {
        match auth {
            HttpAuthentication::BotToken { token } => {
                self.insert("x-bot-token", token.parse().unwrap());
            }
            HttpAuthentication::UserSession {
                user_id,
                session_token,
            } => {
                self.insert("x-session-token", session_token.parse().unwrap());
                self.insert("x-user-id", user_id.as_ref().parse().unwrap());
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
}

macro_rules! ep {
    ($self:expr, $ep:literal $($args:tt)*) => {
        format!(concat!("{}", $ep), $self.api_root, $($args)*)
    }
}

macro_rules! autumn_tag_upload {
    ($self:expr, $tag:expr) => {
        format!("{}/{}", $self.revolt_config.features.autumn.url(), $tag)
    };
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
        let mut default_headers = HeaderMap::new().auth(&auth.into());
        default_headers.insert(reqwest::header::ACCEPT, HeaderValue::from_static("*/*"));
        let client = reqwest::Client::builder()
            .default_headers(default_headers)
            .build()
            .unwrap();
        let revolt_config = Self::get_revolt_config(&client, api_root).await?;
        Ok(Self {
            client,
            api_root: api_root.to_string(),
            revolt_config,
        })
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
    pub async fn open_dm(&self, user_id: UserId) -> Result<Channel> {
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
            .client
            .get(ep!(self, "/users/relationships"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Fetches your relationship with the given user
    pub async fn fetch_relationship(&self, user_id: UserId) -> Result<Relationship> {
        Ok(self
            .client
            .get(ep!(self, "/users/{}/relationship" user_id))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Sends or accepts a friend request to / from the user with given username
    pub async fn send_friend_request(&self, username: &str) -> Result<NewRelationshipResponse> {
        Ok(self
            .client
            .put(ep!(self, "/users/{}/friend" username))
            .send()
            .await?
            .error_for_status()?
            .json::<NewRelationshipResponse>()
            .await?)
    }

    /// Denies a friend request
    pub async fn deny_friend_request(&self, username: &str) -> Result<NewRelationshipResponse> {
        Ok(self
            .client
            .delete(ep!(self, "/users/{}/friend" username))
            .send()
            .await?
            .error_for_status()?
            .json::<NewRelationshipResponse>()
            .await?)
    }

    /// Blocks an user
    pub async fn block(&self, user_id: UserId) -> Result<NewRelationshipResponse> {
        Ok(self
            .client
            .put(ep!(self, "/users/{}/block" user_id))
            .send()
            .await?
            .error_for_status()?
            .json::<NewRelationshipResponse>()
            .await?)
    }

    /// Unblocks an user
    pub async fn unblock(&self, user_id: UserId) -> Result<NewRelationshipResponse> {
        Ok(self
            .client
            .delete(ep!(self, "/users/{}/block" user_id))
            .send()
            .await?
            .error_for_status()?
            .json::<NewRelationshipResponse>()
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
            remove: Option<ChannelField>,
        }

        self.client
            .patch(ep!(self, "/channels/{}" channel_id))
            .json(&PatchChannelRequest {
                name,
                description,
                icon,
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
    pub async fn set_role_permissions(
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
    pub async fn set_default_permissions(
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

    /// Creates a group
    pub async fn create_group(
        &self,
        name: String,
        description: Option<String>,
        nonce: String,
        users: Option<&[UserId]>,
    ) -> Result<Channel> {
        #[derive(serde::Serialize)]
        struct CreateGroupRequest<'a> {
            name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            nonce: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            users: Option<&'a [UserId]>,
        }
        Ok(self
            .client
            .post(ep!(self, "/channels/create"))
            .json(&CreateGroupRequest {
                name,
                description,
                nonce,
                users,
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

    // TODO: groups

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
        nonce: String,
    ) -> Result<Server> {
        #[derive(serde::Serialize)]
        struct CreateServerRequest {
            name: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            nonce: String,
        }
        Ok(self
            .client
            .post(ep!(self, "/servers/create"))
            .json(&CreateServerRequest {
                name,
                description,
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
        self.client
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

    // TODO server permissions
    // TODO roles
    // TODO bots
    // TODO invites
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
pub struct NewRelationshipResponse {
    pub status: RelationshipStatus,
}

mod utils {
    use robespierre_models::{
        autumn::AttachmentId,
        channels::{ChannelPermissions, ReplyData},
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
}
