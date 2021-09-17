use robespierre_models::{
    id::UserId,
    users::{Relationship, RelationshipStatus},
};

use super::impl_prelude::*;

impl Http {
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
}

#[derive(serde::Deserialize, Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct SingleRelationshipResponse {
    pub status: RelationshipStatus,
}
