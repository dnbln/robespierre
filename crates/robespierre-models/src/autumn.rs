use std::fmt;

/// An autumn tag that you can upload images under.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AutumnTag {
    Attachments,
    Avatars,
    Backgrounds,
    Icons,
    Banners,
}

impl AutumnTag {
    pub fn to_str(self) -> &'static str {
        match self {
            Self::Attachments => "attachments",
            Self::Avatars => "avatars",
            Self::Backgrounds => "backgrounds",
            Self::Icons => "icons",
            Self::Banners => "banners",
        }
    }
}

impl fmt::Display for AutumnTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.to_str().fmt(f)
    }
}
