use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    auth::{
        password::{Password, PasswordError},
        token::{Expiry, Token, TokenPair},
    },
    error::auth::AuthError,
    sessions::repository::{SessionRepository, Validities},
    users::{
        model::{Discriminator, Email, User, Username},
        repository::UserRepository,
    },
};

pub(crate) struct AuthService {
    users: UserRepository,
    sessions: SessionRepository,
}

impl AuthService {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self {
            users: UserRepository::new(pool.clone()),
            sessions: SessionRepository::new(pool),
        }
    }

    pub(crate) fn register(
        &self,
        email: Email,
        password: Password,
        username: Username,
        discriminator: Discriminator,
    ) -> impl Future<Output = Result<User, AuthError>> {
        self.users.create(email, password, username, discriminator)
    }

    pub(crate) async fn login(
        &self,
        email: &Email,
        password: &Password,
        session_title: &str,
    ) -> Result<TokenPair, AuthError> {
        let user = self
            .users
            .find_for_login(email)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;
        password
            .verify(user.password_hash)
            .await
            .map_err(|err| match err {
                PasswordError::InvalidHash => AuthError::InvalidCredentials,
                PasswordError::TaskFailed => AuthError::Internal,
            })?;
        let (_, tokens) = self
            .sessions
            .create(user.id, session_title, Validities::default())
            .await?;

        Ok(tokens)
    }

    pub(crate) fn refresh<'a, E: Expiry>(
        &'a self,
        user_id: &'a Uuid,
        refresh_token: &'a Token<E>,
    ) -> impl Future<Output = Result<TokenPair, AuthError>> + 'a {
        self.sessions.refresh(user_id, refresh_token)
    }

    /*pub(crate) async fn authenticate_access_token(
        &self,
        access_token: &str,
    ) -> Result<AuthenticatedUser, AuthError>{todo!();}*/

    pub(crate) fn logout<E: Expiry>(
        &self,
        refresh_token: &Token<E>,
    ) -> impl Future<Output = Result<(), AuthError>> {
        self.sessions.logout(refresh_token)
    }

    pub(crate) const fn sessions(&self) -> &SessionRepository {
        &self.sessions
    }
}
