use std::result::Result as StdResult;

use reqwest::RequestBuilder;
use robespierre_models::{
    attachments::AutumnFileId,
    channel::{
        Channel, ChannelField, ChannelPermissions, CreateChannelInviteResponse, DmChannel, Message,
        MessageFilter, ReplyData,
    },
    id::{ChannelId, MessageId, RoleId, ServerId, UserId},
    server::{Member, MemberField, PartialMember, PartialServer, Server, ServerField},
    user::{NewRelationshipResponse, Relationship, User, UserEditPatch, UserProfileData},
};

use crate::utils::{PermissionsUpdateRequest, SendMessageRequest};

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("decoding: {0}")]
    Decoding(#[from] serde_json::Error),
}

pub type Result<T = ()> = StdResult<T, HttpError>;

pub enum HttpAuthentication {
    BotToken {
        token: String,
    },
    UserSession {
        user_id: UserId,
        session_token: String,
    },
}
trait AuthExt: Sized {
    fn auth(self, auth: &HttpAuthentication) -> Self;
}

impl AuthExt for RequestBuilder {
    fn auth(self, auth: &HttpAuthentication) -> Self {
        match auth {
            HttpAuthentication::BotToken { token } => self.header("x-bot-token", token),
            HttpAuthentication::UserSession { user_id, session_token } => {
                self.header("x-session-token", session_token).header("x-user-id", format!("{}", user_id))
            }
        }
    }
}

pub struct Http {
    auth: HttpAuthentication,
    client: reqwest::Client,
}

const ROOT_LINK: &str = "https://api.revolt.chat";

macro_rules! ep {
    ($ep:literal $($args:tt)*) => {
        format!(concat!("{}", $ep), ROOT_LINK, $($args)*)
    }
}

impl Http {
    pub fn new(auth: HttpAuthentication) -> Self {
        Self {
            auth,
            client: reqwest::Client::new(),
        }
    }

    pub async fn fetch_user(&self, user_id: UserId) -> Result<User> {
        Ok(self
            .client
            .get(ep!("/users/{}" user_id))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn edit_user(&self, patch: UserEditPatch) -> Result {
        self.client
            .patch(ep!("/users/@me"))
            .auth(&self.auth)
            .json(&patch)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn fetch_user_profile(&self, user_id: UserId) -> Result<UserProfileData> {
        Ok(self
            .client
            .get(ep!("/users/{}/profile" user_id))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn fetch_dm_channels(&self) -> Result<Vec<DmChannel>> {
        Ok(self
            .client
            .get(ep!("/users/dms"))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn open_dm(&self, user_id: UserId) -> Result<DmChannel> {
        Ok(self
            .client
            .get(ep!("/users/{}/dm" user_id))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn fetch_relationships(&self) -> Result<Vec<Relationship>> {
        Ok(self
            .client
            .get(ep!("/users/relationships"))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn fetch_relationship(&self, user_id: UserId) -> Result<Relationship> {
        Ok(self
            .client
            .get(ep!("/users/{}/relationship" user_id))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn send_friend_request(&self, username: &str) -> Result<NewRelationshipResponse> {
        Ok(self
            .client
            .put(ep!("/users/{}/friend" username))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?
            .json::<NewRelationshipResponse>()
            .await?)
    }

    pub async fn deny_friend_request(&self, username: &str) -> Result<NewRelationshipResponse> {
        Ok(self
            .client
            .delete(ep!("/users/{}/friend" username))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?
            .json::<NewRelationshipResponse>()
            .await?)
    }

    pub async fn block(&self, user_id: UserId) -> Result<NewRelationshipResponse> {
        Ok(self
            .client
            .put(ep!("/users/{}/block" user_id))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?
            .json::<NewRelationshipResponse>()
            .await?)
    }

    pub async fn unblock(&self, user_id: UserId) -> Result<NewRelationshipResponse> {
        Ok(self
            .client
            .delete(ep!("/users/{}/block" user_id))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?
            .json::<NewRelationshipResponse>()
            .await?)
    }

    pub async fn fetch_channel(&self, channel_id: ChannelId) -> Result<Channel> {
        Ok(self
            .client
            .get(ep!("/channels/{}" channel_id))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn edit_channel(
        &self,
        channel_id: ChannelId,
        name: Option<String>,
        description: Option<String>,
        icon: Option<AutumnFileId>,
        remove: Option<ChannelField>,
    ) -> Result {
        #[derive(serde::Serialize)]
        struct PatchChannelRequest {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            icon: Option<AutumnFileId>,
            #[serde(skip_serializing_if = "Option::is_none")]
            remove: Option<ChannelField>,
        }

        self.client
            .patch(ep!("/channels/{}" channel_id))
            .auth(&self.auth)
            .json(&PatchChannelRequest { name, description, icon, remove })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn close_channel(&self, channel_id: ChannelId) -> Result {
        self.client
            .delete(ep!("/channels/{}" channel_id))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn create_invite(
        &self,
        channel_id: ChannelId,
    ) -> Result<CreateChannelInviteResponse> {
        Ok(self
            .client
            .post(ep!("/channels/{}/invites" channel_id))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn set_role_permissions(
        &self,
        channel_id: ChannelId,
        role_id: RoleId,
        permissions: ChannelPermissions,
    ) -> Result {
        self.client
            .put(ep!("/channels/{}/permissions/{}" channel_id, role_id))
            .auth(&self.auth)
            .json(&PermissionsUpdateRequest { permissions })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn set_default_permissions(
        &self,
        channel_id: ChannelId,
        permissions: ChannelPermissions,
    ) -> Result {
        self.client
            .put(ep!("/channels/{}/permissions/default" channel_id))
            .auth(&self.auth)
            .json(&PermissionsUpdateRequest { permissions })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn send_message(
        &self,
        channel_id: ChannelId,
        content: impl AsRef<str>,
        nonce: impl AsRef<str>,
        attachments: Vec<AutumnFileId>,
        replies: Vec<ReplyData>,
    ) -> Result<Message> {
        Ok(self
            .client
            .post(ep!("/channels/{}/messages" channel_id))
            .auth(&self.auth)
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

    pub async fn fetch_messages(
        &self,
        channel: ChannelId,
        filter: MessageFilter,
    ) -> Result<FetchMessagesResult> {
        let v = self
            .client
            .get(ep!("/channels/{}/messages" channel))
            .auth(&self.auth)
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

    pub async fn fetch_message(&self, channel: ChannelId, message: MessageId) -> Result<Message> {
        Ok(self
            .client
            .get(ep!("/channels/{}/messages/{}" channel, message))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

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
            .patch(ep!("/channels/{}/messages/{}" channel, message))
            .auth(&self.auth)
            .json(&MessagePatch { content })
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn delete_message(&self, channel: ChannelId, message: MessageId) -> Result {
        self.client
            .delete(ep!("/channels/{}/messages/{}" channel, message))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    // TODO: groups

    pub async fn fetch_server(&self, server: ServerId) -> Result<Server> {
        Ok(self
            .client
            .get(ep!("/servers/{}" server))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

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
            .patch(ep!("/servers/{}" server_id))
            .auth(&self.auth)
            .json(&ServerPatchRequest { server, remove })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn delete_server(&self, server_id: ServerId) -> Result {
        self.client
            .delete(ep!("/servers/{}" server_id))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    // TODO create server
    // TODO create channel
    // TODO fetch invites
    // TODO mark server as read

    pub async fn fetch_member(&self, server_id: ServerId, user_id: UserId) -> Result<Member> {
        Ok(self
            .client
            .get(ep!("/servers/{}/members/{}" server_id, user_id))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

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
            .patch(ep!("/servers/{}/members/{}" server_id, user_id))
            .auth(&self.auth)
            .json(&PatchMemberRequest { member, remove })
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn kick_member(&self, server_id: ServerId, user_id: UserId) -> Result {
        self.client
            .delete(ep!("/servers/{}/members/{}" server_id, user_id))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn fetch_all_members(&self, server_id: ServerId) -> Result<FetchMembersResult> {
        Ok(self
            .client
            .get(ep!("/servers/{}/members" server_id))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

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
            .put(ep!("/servers/{}/bans/{}" server_id, user_id))
            .auth(&self.auth)
            .json(&BanRequest { reason })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn unban_user(&self, server_id: ServerId, user_id: UserId) -> Result {
        self.client
            .delete(ep!("/servers/{}/bans/{}" server_id, user_id))
            .auth(&self.auth)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    // TODO fetch bans

    // TODO server permissions
    // TODO roles
    // TODO bots
    // TODO invites
    // TODO sync
}

#[derive(serde::Deserialize)]
pub struct FetchMessagesResult {
    pub messages: Vec<Message>,
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(default)]
    pub members: Vec<Member>,
}

#[derive(serde::Deserialize)]
pub struct FetchMembersResult {
    pub users: Vec<User>,
    pub members: Vec<Member>,
}

mod utils {
    use robespierre_models::{
        attachments::AutumnFileId,
        channel::{ChannelPermissions, ReplyData},
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
        pub attachments: Vec<AutumnFileId>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub replies: Vec<ReplyData>,
    }
}
