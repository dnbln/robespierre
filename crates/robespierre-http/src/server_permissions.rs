use robespierre_models::{channels::ChannelPermissions, id::{RoleId, ServerId}, servers::{PermissionTuple, ServerPermissions}};

use super::impl_prelude::*;

impl Http {
    pub async fn set_role_permissions(
        &self,
        server: ServerId,
        role: RoleId,
        permissions: PermissionTuple,
    ) -> Result {
        let (sp, cp) = permissions;

        self.client
            .put(ep!(self, "/servers/{}/permissions/{}" server, role))
            .json(&SetPermissionsRequest {
                server: sp,
                channel: cp,
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn set_default_role_permissions(
        &self,
        server: ServerId,
        permissions: PermissionTuple,
    ) -> Result {
        let (sp, cp) = permissions;

        self.client
            .put(ep!(self, "/servers/{}/permissions/default" server))
            .json(&SetPermissionsRequest {
                server: sp,
                channel: cp,
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn create_role(&self, server: ServerId, name: &str) -> Result<RoleIdAndPermissions> {
        #[derive(serde::Serialize)]
        struct CreateRoleRequest<'a> {
            name: &'a str,
        }

        Ok(self
            .client
            .post(ep!(self, "/servers/{}/roles" server))
            .json(&CreateRoleRequest { name })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    // TODO: edit role

    pub async fn delete_role(&self, server: ServerId, role: RoleId) -> Result {
        self.client
            .delete(ep!(self, "/servers/{}/roles/{}" server, role))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(serde::Deserialize)]
pub struct RoleIdAndPermissions {
    pub id: RoleId,
    pub permissions: PermissionTuple,
}

#[derive(serde::Serialize)]
pub struct SetPermissionsRequest {
    pub server: ServerPermissions,
    pub channel: ChannelPermissions,
}
