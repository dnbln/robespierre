use robespierre_models::{channels::{Channel, DirectMessageChannel}, id::UserId};

use super::impl_prelude::*;

impl Http {
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
}
