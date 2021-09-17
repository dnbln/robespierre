use robespierre_models::core::RevoltConfiguration;

use crate::{Http, Result};

impl Http {
    pub(crate) async fn get_revolt_config(
        client: &reqwest::Client,
        root_url: &str,
    ) -> Result<RevoltConfiguration> {
        Ok(client
            .get(root_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}
