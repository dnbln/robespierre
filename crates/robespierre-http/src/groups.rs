use robespierre_models::{
    channels::Channel,
    id::{ChannelId, UserId},
    users::User,
};

use super::impl_prelude::*;

impl Http {
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
}
