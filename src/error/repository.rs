use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]

pub(crate) enum RepositoryError {
    #[error("Item not found")]
    ItemNotFound,

    #[error("List not found")]
    ListNotFound,

    #[error("Already exists")]
    Duplicate,

    #[error("Database operation failed")]
    Database(#[from] sqlx::Error),
}

impl RepositoryError {
    pub fn map_insert(err: sqlx::Error) -> Self {
        let Some(db_err) = err.as_database_error() else {
            return Self::Database(err);
        };
        if db_err.is_unique_violation() {
            return Self::Duplicate;
        }
        if db_err.is_foreign_key_violation() {
            return Self::ListNotFound;
        }
        Self::Database(err)
    }
}

impl IntoResponse for RepositoryError {
    fn into_response(self) -> Response {
        match self {
            Self::ItemNotFound | Self::ListNotFound => StatusCode::NOT_FOUND.into_response(),
            Self::Duplicate => StatusCode::CONFLICT.into_response(),
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
