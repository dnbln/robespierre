#[derive(serde::Deserialize, Clone)]
pub struct RevoltInstanceData {
    pub revolt: String,
    pub features: RevoltInstanceFeatures,
    pub ws: String,
    pub app: String,
    pub vapid: String,
}

#[derive(serde::Deserialize, Clone)]
#[serde(transparent)]
pub struct Autumn(EnabledUrl);

impl Autumn {
    pub fn is_enabled(&self) -> bool {
        self.0.enabled
    }

    pub fn url(&self) -> &String {
        &self.0.url
    }
}

#[derive(serde::Deserialize, Clone)]
#[serde(transparent)]
pub struct January(EnabledUrl);

impl January {
    pub fn is_enabled(&self) -> bool {
        self.0.enabled
    }

    pub fn url(&self) -> &String {
        &self.0.url
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct EnabledUrl {
    pub enabled: bool,
    pub url: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct EnabledUrlWs {
    pub enabled: bool,
    pub url: String,
    pub ws: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct RevoltInstanceFeatures {
    // pub registration: bool,
    pub captcha: CaptchaInfo,
    pub email: bool,
    pub invite_only: bool,
    pub autumn: Autumn,
    pub january: January,
    pub voso: EnabledUrlWs,
}

#[derive(serde::Deserialize, Clone)]
pub struct CaptchaInfo {
    pub enabled: bool,
    pub key: String,
}