use robespierre_models::{
    auth::{Account, Session, SessionInfo},
    id::SessionId,
};

use super::impl_prelude::*;

impl Http {
    // account
    pub async fn fetch_account(&self) -> Result<Account> {
        Ok(self
            .client_user_session_auth_type()
            .get(ep!(self, "/auth/account"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn create_account(
        email: &str,
        password: &str,
        invite: Option<&str>,
        captcha: Option<&str>,
    ) -> Result {
        #[derive(serde::Serialize)]
        struct CreateAccountRequest<'a> {
            email: &'a str,
            password: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            invite: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            captcha: Option<&'a str>,
        }

        reqwest::Client::new()
            .post(ep!(
                api_root = "https://api.revolt.chat",
                "/auth/account/create"
            ))
            .json(&CreateAccountRequest {
                email,
                password,
                invite,
                captcha,
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn resend_verification(email: &str, captcha: Option<&str>) -> Result {
        #[derive(serde::Serialize)]
        struct ResendVerificationRequest<'a> {
            email: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            captcha: Option<&'a str>,
        }

        reqwest::Client::new()
            .post(ep!(
                api_root = "https://api.revolt.chat",
                "/auth/account/reverify"
            ))
            .json(&ResendVerificationRequest { email, captcha })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn verify_email(code: &str) -> Result {
        reqwest::Client::new()
            .post(ep!(
                api_root = "https://api.revolt.chat",
                "/auth/account/verify/{}" code
            ))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn send_password_reset(email: &str, captcha: Option<&str>) -> Result {
        #[derive(serde::Serialize)]
        struct SendPasswordResetRequest<'a> {
            email: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            captcha: Option<&'a str>,
        }

        reqwest::Client::new()
            .post(ep!(
                api_root = "https://api.revolt.chat",
                "/auth/account/reset_password"
            ))
            .json(&SendPasswordResetRequest { email, captcha })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn password_reset(password: &str, token: &str) -> Result {
        #[derive(serde::Serialize)]
        struct PasswordResetRequest<'a> {
            password: &'a str,
            token: &'a str,
        }

        reqwest::Client::new()
            .patch(ep!(
                api_root = "https://api.revolt.chat",
                "/auth/account/reset_password"
            ))
            .json(&PasswordResetRequest { password, token })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn change_password(&self, password: &str, current_password: &str) -> Result {
        #[derive(serde::Serialize)]
        struct ChangePasswordRequest<'a> {
            password: &'a str,
            current_password: &'a str,
        }

        self.client_user_session_auth_type()
            .post(ep!(self, "/auth/account/change/password"))
            .json(&ChangePasswordRequest {
                password,
                current_password,
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn change_email(&self, email: &str, current_password: &str) -> Result {
        #[derive(serde::Serialize)]
        struct ChangeEmailRequest<'a> {
            email: &'a str,
            current_password: &'a str,
        }

        self.client_user_session_auth_type()
            .post(ep!(self, "/auth/account/change/email"))
            .json(&ChangeEmailRequest {
                email,
                current_password,
            })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn login(
        email: &str,
        password: Option<&str>,
        challange: Option<&str>,
        friendly_name: Option<&str>,
        captcha: Option<&str>,
    ) -> Result<Session> {
        #[derive(serde::Serialize)]
        struct LoginRequest<'a> {
            email: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            password: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            challange: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            friendly_name: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            captcha: Option<&'a str>,
        }

        Ok(reqwest::Client::new()
            .post(ep!(
                api_root = "https://api.revolt.chat",
                "/auth/session/login"
            ))
            .json(&LoginRequest {
                email,
                password,
                challange,
                friendly_name,
                captcha,
            })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn logout(self) -> Result {
        self.client_user_session_auth_type()
            .delete(ep!(self, "/auth/session/logout"))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn edit_session(&self, session: SessionId, friendly_name: &str) -> Result {
        #[derive(serde::Serialize)]
        struct EditSessionRequest<'a> {
            friendly_name: &'a str,
        }

        self.client_user_session_auth_type()
            .patch(ep!(self, "/auth/session/{}" session))
            .json(&EditSessionRequest { friendly_name })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn delete_session(&self, session: SessionId) -> Result {
        self.client_user_session_auth_type()
            .delete(ep!(self, "/auth/session/{}" session))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn fetch_sessions(&self) -> Result<Vec<SessionInfo>> {
        Ok(self
            .client_user_session_auth_type()
            .get(ep!(self, "/auth/session/all"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn delete_all_sessions(&self, revoke_self: bool) -> Result {
        self.client_user_session_auth_type()
            .delete(ep!(self, "/auth/session/all"))
            .query(&[("revoke_self", revoke_self)])
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
