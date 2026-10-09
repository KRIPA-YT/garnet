use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::error::{auth::AuthError, repository::RepositoryError};

#[derive(Error, Debug)]
#[non_exhaustive]
pub(crate) enum AppError {
    #[error("AuthError: $1")]
    Auth(#[from] AuthError),
    #[error("RepositoryError: $1")]
    RepositoryError(#[from] RepositoryError),
    #[error("Bad request")]
    BadRequest,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::BadRequest => StatusCode::BAD_REQUEST.into_response(),
            Self::Auth(err) => err.into_response(),
            Self::RepositoryError(err) => err.into_response(),
        }
    }
}
