use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub(crate) enum AuthError {
    #[error("Provided credentials are invalid")]
    InvalidCredentials,
    #[error("One or more parameters already exist")]
    Conflict,
    #[error("Database: $1")]
    Database(#[from] sqlx::error::Error),
    #[error("")]
    Internal,
}
