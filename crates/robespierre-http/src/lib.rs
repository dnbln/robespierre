use std::result::Result as StdResult;

use reqwest::RequestBuilder;
use robespierre_models::{
    attachments::AutumnFileId,
    channel::{CreateChannelInviteResponse, DmChannel, Message, Permissions, ReplyData},
    id::{ChannelId, RoleId, UserId},
    user::{NewRelationshipResponse, Relationship, User, UserEditPatch, UserProfileData},
};

use crate::utils::{PermissionsUpdateRequest, SendMessageRequest};

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
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
            HttpAuthentication::UserSession { .. } => {
                unimplemented!("authentication with user session not implemented yet")
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

    pub async fn fetch_user(&self, user_id: &UserId) -> Result<User> {
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

    pub async fn fetch_user_profile(&self, user_id: &UserId) -> Result<UserProfileData> {
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

    pub async fn open_dm(&self, user_id: &UserId) -> Result<DmChannel> {
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

    pub async fn fetch_relationship(&self, user_id: &UserId) -> Result<Relationship> {
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

    pub async fn block(&self, user_id: &UserId) -> Result<NewRelationshipResponse> {
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

    pub async fn unblock(&self, user_id: &UserId) -> Result<NewRelationshipResponse> {
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

    pub async fn close_channel(&self, channel_id: &ChannelId) -> Result {
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
        channel_id: &ChannelId,
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
        channel_id: &ChannelId,
        role_id: &RoleId,
        permissions: Permissions,
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
        channel_id: &ChannelId,
        permissions: Permissions,
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
        channel_id: &ChannelId,
        content: String,
        nonce: String,
        attachments: Vec<AutumnFileId>,
        replies: Vec<ReplyData>,
    ) -> Result<Message> {
        Ok(self
            .client
            .post(ep!("/channels/{}/messages" channel_id))
            .auth(&self.auth)
            .json(&SendMessageRequest {
                content,
                nonce,
                attachments,
                replies,
            })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}

mod utils {
    use robespierre_models::{
        attachments::AutumnFileId,
        channel::{Permissions, ReplyData},
    };

    use serde::Serialize;

    #[derive(Serialize)]
    pub struct PermissionsUpdateRequest {
        pub permissions: Permissions,
    }

    #[derive(Serialize)]
    pub struct SendMessageRequest {
        pub content: String,
        pub nonce: String,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub attachments: Vec<AutumnFileId>,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub replies: Vec<ReplyData>,
    }
}
