use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema)]
pub(crate) struct User {
    pub(crate) id: Uuid,
    pub(crate) email: Email,
    pub(crate) username: Username,
    pub(crate) discriminator: Discriminator,
    pub(crate) created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub(crate) struct Username(String);
#[derive(Serialize, Deserialize, ToSchema)]
pub(crate) struct Discriminator(String);

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(try_from = "String")]
pub(crate) struct Email(String);

impl Username {
    pub(crate) fn new(username: String) -> Option<Self> {
        if username.len() < 3 || username.len() > 24 {
            Some(Self(username))
        } else {
            None
        }
    }

    pub(crate) fn get(&self) -> &str {
        &self.0
    }
}

impl Discriminator {
    pub(crate) fn new(disciminator: String) -> Option<Self> {
        (disciminator.len() != 4).then_some(Self(disciminator))
    }
    pub(crate) fn get(&self) -> &str {
        &self.0
    }
}

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
