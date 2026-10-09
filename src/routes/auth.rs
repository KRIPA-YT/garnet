use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use axum_extra::{
    TypedHeader,
    headers::{
        Authorization,
        authorization::{Basic, Bearer},
    },
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    app::AppState,
    auth::{
        password::Password,
        token::{Token, TokenPair},
    },
    error::app::AppError,
    sessions::model::Session,
    users::model::{Discriminator, Email, User, Username},
};

#[derive(Deserialize, ToSchema)]
pub(crate) struct RegisterRequest {
    pub username: String,
    pub discriminator: String,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct AuthResponse {
    pub access_token: String,
    pub access_expires_in: u64,
    pub refresh_token: String,
    pub refresh_expires_in: u64,
}

impl AuthResponse {
    pub(crate) fn with_token_pair(pair: &TokenPair) -> Self {
        Self {
            access_token: pair.access.base64_encode(),
            access_expires_in: (pair.access.expiry.signed_duration_since(Utc::now()))
                .num_seconds()
                .cast_unsigned(),
            refresh_token: pair.refresh.base64_encode(),
            refresh_expires_in: (pair.refresh.expiry.signed_duration_since(Utc::now()))
                .num_seconds()
                .cast_unsigned(),
        }
    }
}

#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "Auth",
    security(
        ("basic_auth" = [])
    ),
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered", body = User),
        (status = 400, description = "Input data is malformed or is not conforming to guidelines"),
        (status = 409, description = "User already exists"),
        (status = 500, description = "Internal server error")
    )
)]
pub(crate) async fn register(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Basic>>,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    let email = Email::new(authorization.username().to_string()).ok_or(AppError::BadRequest)?;
    let password =
        Password::new(authorization.password().to_string()).ok_or(AppError::BadRequest)?;
    let discriminator = Discriminator::new(payload.discriminator).ok_or(AppError::BadRequest)?;
    let username = Username::new(payload.username).ok_or(AppError::BadRequest)?;

    state
        .auth
        .register(email, password, username, discriminator)
        .await
        .map(Json)
        .map_err(AppError::from)
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct LoginRequest {
    pub title: String,
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "Auth",
    security(
        ("basic_auth" = [])
    ),
    request_body = LoginRequest,
    responses(
        (status = 200, description = "User logged in", body = AuthResponse),
        (status = 400, description = "Input data is malformed or is not conforming to guidelines"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub(crate) async fn login(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Basic>>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let email = Email::new(authorization.username().to_string()).ok_or(AppError::BadRequest)?;
    let password =
        Password::new(authorization.password().to_string()).ok_or(AppError::BadRequest)?;

    state
        .auth
        .login(&email, &password, &payload.title)
        .await
        .map(|token_pair| Json(AuthResponse::with_token_pair(&token_pair)))
        .map_err(AppError::from)
}

#[derive(Deserialize, ToSchema)]
pub(crate) struct RefreshQuery {
    user_id: Uuid,
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    tag = "Auth",
    security(
        ("bearer_auth" = [])
    ),
    params(
        ("user_id" = Uuid, Query, description = "User id to refresh the token for")
    ),
    responses(
        (status = 200, description = "Token refreshed", body = AuthResponse),
        (status = 400, description = "Malformed bearer token"),
        (status = 401, description = "Invalid refresh token"),
        (status = 500, description = "Internal server error"),
    )
)]
pub(crate) async fn refresh(
    State(app_state): State<AppState>,
    Query(query): Query<RefreshQuery>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    app_state
        .auth
        .refresh(
            &query.user_id,
            &Token::from_base64(authorization.token()).ok_or(AppError::BadRequest)?,
        )
        .await
        .map(|token_pair| Json(AuthResponse::with_token_pair(&token_pair)))
        .map_err(AppError::from)
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "Auth",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 204, description = "User logged out"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub(crate) async fn logout(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    state
        .auth
        .logout(&Token::unlimited(authorization.token().to_owned()))
        .await
}

#[utoipa::path(
    post,
    path = "/auth/session",
    tag = "Auth",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Current session", body = Session),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub(crate) async fn get_session(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    state
        .auth
        .sessions()
        .get(&Token::unlimited(authorization.token().to_owned()))
        .await
        .map(Json)
}
