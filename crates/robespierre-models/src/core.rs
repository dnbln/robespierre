/// Data about a revolt instance obtained by
/// making a `GET /` on the api.
#[derive(serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RevoltConfiguration {
    pub revolt: String,
    pub features: RevoltInstanceFeatures,
    pub ws: String,
    pub app: String,
    pub vapid: String,
}

/// Data about Autumn (file server microservice).
#[derive(serde::Deserialize, Clone)]
#[serde(transparent)]
#[serde(deny_unknown_fields)]
pub struct Autumn(EnabledUrl);

impl Autumn {
    /// Is Autumn enabled?
    pub fn is_enabled(&self) -> bool {
        self.0.enabled
    }

    /// Get the url
    pub fn url(&self) -> &String {
        &self.0.url
    }
}

/// Data about January (image proxy and embed generator).
#[derive(serde::Deserialize, Clone)]
#[serde(transparent)]
#[serde(deny_unknown_fields)]
pub struct January(EnabledUrl);

impl January {
    /// Is January enabled?
    pub fn is_enabled(&self) -> bool {
        self.0.enabled
    }

    /// Get the url
    pub fn url(&self) -> &String {
        &self.0.url
    }
}

/// Data about Voso (legacy voice server).
#[derive(serde::Deserialize, Clone)]
#[serde(transparent)]
#[serde(deny_unknown_fields)]
pub struct Voso(EnabledUrlWs);

impl Voso {
    /// Is voso enabled?
    pub fn is_enabled(&self) -> bool {
        self.0.enabled
    }

    /// Get the url
    pub fn url(&self) -> &String {
        &self.0.url
    }

    /// Get the ws url.
    pub fn ws_url(&self) -> &String {
        &self.0.ws
    }
}

#[derive(serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct EnabledUrl {
    enabled: bool,
    url: String,
}

#[derive(serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct EnabledUrlWs {
    enabled: bool,
    url: String,
    ws: String,
}

/// Features

#[derive(serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RevoltInstanceFeatures {
    // pub registration: bool,
    pub captcha: CaptchaInfo,
    /// Uses email verification?
    pub email: bool,
    /// Is invite only?
    pub invite_only: bool,
    /// Autumn (file server microservice).
    pub autumn: Autumn,
    /// January (image proxy and embed generator)
    pub january: January,
    /// Voso (legacy voice server).
    pub voso: Voso,
}

/// Captcha feature
#[derive(serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct CaptchaInfo {
    /// Whether it is enabled or not
    pub enabled: bool,
    /// The captcha key
    pub key: String,
}
