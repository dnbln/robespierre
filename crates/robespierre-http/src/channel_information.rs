use serde::Serialize;

use robespierre_models::{
    autumn::AttachmentId,
    channels::{Channel, ChannelField, ChannelPermissions, CreateChannelInviteResponse},
    id::{ChannelId, RoleId},
};

use super::impl_prelude::*;

impl Http {
    /// Gets the channel given the id
    pub async fn fetch_channel(&self, channel_id: ChannelId) -> Result<Channel> {
        Ok(self
            .client
            .get(ep!(self, "/channels/{}" channel_id))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Edits the channel given by id
    pub async fn edit_channel(
        &self,
        channel_id: ChannelId,
        name: Option<String>,
        description: Option<String>,
        icon: Option<AttachmentId>,
        nsfw: Option<bool>,
        remove: Option<ChannelField>,
    ) -> Result {
        #[derive(serde::Serialize)]
        struct PatchChannelRequest {
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            icon: Option<AttachmentId>,
            #[serde(skip_serializing_if = "Option::is_none")]
            nsfw: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            remove: Option<ChannelField>,
        }

        self.client
            .patch(ep!(self, "/channels/{}" channel_id))
            .json(&PatchChannelRequest {
                name,
                description,
                icon,
                nsfw,
                remove,
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Closes a channel / leaves group
    pub async fn close_channel(&self, channel_id: ChannelId) -> Result {
        self.client
            .delete(ep!(self, "/channels/{}" channel_id))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Creates an invite
    pub async fn create_invite(
        &self,
        channel_id: ChannelId,
    ) -> Result<CreateChannelInviteResponse> {
        Ok(self
            .client
            .post(ep!(self, "/channels/{}/invites" channel_id))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    /// Sets role permissions
    pub async fn set_channel_role_permissions(
        &self,
        channel_id: ChannelId,
        role_id: RoleId,
        permissions: ChannelPermissions,
    ) -> Result {
        self.client
            .put(ep!(self, "/channels/{}/permissions/{}" channel_id, role_id))
            .json(&PermissionsUpdateRequest { permissions })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Sets default permissions
    pub async fn set_channel_default_role_permissions(
        &self,
        channel_id: ChannelId,
        permissions: ChannelPermissions,
    ) -> Result {
        self.client
            .put(ep!(self, "/channels/{}/permissions/default" channel_id))
            .json(&PermissionsUpdateRequest { permissions })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(Serialize)]
pub struct PermissionsUpdateRequest {
    pub permissions: ChannelPermissions,
}
