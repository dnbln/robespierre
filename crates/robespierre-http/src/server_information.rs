use robespierre_models::{
    channels::{ChannelInviteCode, ServerChannelType},
    id::{ChannelId, ServerId, UserId},
    servers::{PartialServer, Server, ServerField},
};

use super::impl_prelude::*;

impl Http {
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
