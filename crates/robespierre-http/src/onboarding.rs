use super::impl_prelude::*;

impl Http {
    // onboarding
    pub async fn get_onboarding(&self) -> Result<OnboardingStatus> {
        Ok(self
            .client_user_session_auth_type()
            .get(ep!(self, "/onboard/hello"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn complete_onboarding(&self, username: &str) -> Result {
        #[derive(serde::Serialize)]
        struct CompleteOnboardingRequest<'a> {
            username: &'a str,
        }

        self.client_user_session_auth_type()
            .post(ep!(self, "/onboard/complete"))
            .json(&CompleteOnboardingRequest { username })
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(serde::Deserialize)]
pub struct OnboardingStatus {
    pub onboarding: bool,
}
