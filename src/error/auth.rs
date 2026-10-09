use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
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

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED.into_response(),
            Self::Conflict => StatusCode::CONFLICT.into_response(),
            _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
