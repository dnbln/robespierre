use robespierre_models::{id::UserId, users::{Profile, User, UserEditPatch}};

use super::impl_prelude::*;

impl Http {
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
}
