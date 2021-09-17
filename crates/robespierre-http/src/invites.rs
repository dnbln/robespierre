use robespierre_models::{channels::Channel, invites::RetrievedInvite, servers::Server};

use super::impl_prelude::*;

impl Http {
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
