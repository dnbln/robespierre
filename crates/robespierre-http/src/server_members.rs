use robespierre_models::{
    autumn::Attachment,
    id::{ServerId, UserId},
    servers::{Ban, Member, MemberField, PartialMember},
    users::User,
};

use super::impl_prelude::*;

impl Http {
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
