use argon2::{Argon2, PasswordHasher};

pub(crate) struct Password(String);
impl Password {
    pub(crate) fn new(password: String) -> Option<Self> {
        Some(Self(password)) // TODO: Implement server side password verification logic
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
}
