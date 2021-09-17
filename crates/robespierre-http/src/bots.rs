use robespierre_models::{bot::{Bot, BotField, PublicBot}, id::{ChannelId, ServerId, UserId}, users::User};

use super::impl_prelude::*;

impl Http {
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
    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            InviteBotTarget::Server(server) => {
                #[derive(serde::Serialize)]
                struct TargetServer {
                    server: ServerId,
                }

                TargetServer { server }.serialize(serializer)
            }
            InviteBotTarget::Group(group) => {
                #[derive(serde::Serialize)]
                struct TargetGroup {
                    group: ChannelId,
                }

                TargetGroup { group }.serialize(serializer)
            }
        }
    }
}