use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub(crate) struct User {
    id: Uuid,
    email: Email,
    username: String,
    discriminator: String,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub(crate) struct Email(String);

impl Email {
    pub(crate) fn new(email: String) -> Option<Self> {
        if email.contains('@') {
            Some(Self(email))
        } else {
            None
        }
    }

    pub(crate) fn get(&self) -> &str {
        &self.0
    }

    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

pub(crate) enum EmailTryFromError {
    Malformed,
}

impl std::fmt::Display for EmailTryFromError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

impl TryFrom<String> for Email {
    type Error = EmailTryFromError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value).ok_or(EmailTryFromError::Malformed)
    }
}
