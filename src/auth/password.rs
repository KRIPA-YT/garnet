use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier as _};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum PasswordError {
    #[error("hash is invalid")]
    InvalidHash,
    #[error("verification task failed")]
    TaskFailed,
}

pub(crate) struct Password(String);
impl Password {
    pub(crate) fn new(password: String) -> Option<Self> {
        const MIN_LEN: usize = 12;
        const MAX_LEN: usize = 128;

        let len = password.chars().count();

        if !(MIN_LEN..=MAX_LEN).contains(&len) {
            return None;
        }

        if password.chars().any(char::is_control) {
            return None;
        }

        Some(Self(password))
    }

    pub(crate) fn get(&self) -> &str {
        &self.0
    }

    #[allow(unused)]
    pub(crate) fn into_inner(self) -> String {
        self.0
    }

    pub(crate) async fn hash(&self) -> Option<String> {
        let password = self.0.clone();
        tokio::task::spawn_blocking(move || {
            Argon2::default()
                .hash_password(password.as_bytes())
                .map(|hash| hash.to_string())
        })
        .await
        .ok()?
        .ok()
    }
    pub(crate) async fn verify(&self, hash: String) -> Result<bool, PasswordError> {
        let password = self.get().to_owned();
        let password_hash = hash;

        tokio::task::spawn_blocking(move || {
            let parsed_hash =
                PasswordHash::new(&password_hash).map_err(|_| PasswordError::InvalidHash)?;

            Ok(Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok())
        })
        .await
        .map_err(|_| PasswordError::TaskFailed)?
    }
}
