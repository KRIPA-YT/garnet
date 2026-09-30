use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{
        Authorization,
        authorization::{Basic, Bearer},
    },
};
use base64::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    AppState,
    auth::{
        password::Password,
        session::{Session, SessionError, SessionParams},
    },
    user::model::Email,
};

#[derive(Deserialize, ToSchema)]
pub(crate) struct RegisterRequest {
    pub username: String,
    pub discriminator: String,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub(crate) struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
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
        (status = 201, description = "User registered", body = UserResponse),
        (status = 400, description = "Input data is malformed or is not conforming to guidelines"),
        (status = 409, description = "User already exists"),
        (status = 500, description = "Internal server error")
    )
)]
#[allow(clippy::result_large_err)]
pub(crate) async fn register(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Basic>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Response, Response> {
    let email = Email::new(authorization.username().to_string())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Malformed email").into_response())?;
    let password = Password::new(authorization.password().to_string())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Malformed password").into_response())?;
    if payload.discriminator.len() != 4 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Discriminator has to be 4 characters long",
        )
            .into_response());
    }
    if payload.username.len() < 3 || payload.username.len() > 24 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Username has to be between 3 and 24 characters long",
        )
            .into_response());
    }

    let user = sqlx::query_as!(
        UserResponse,
        r#"
    INSERT INTO users (username, discriminator, email, password_hash) VALUES ($1, $2, $3, $4) RETURNING id, email, created_at
    "#,
        payload.username,
        payload.discriminator,
        email.get(),
        password.hash().await
    ).fetch_one(&state.pool).await.map_err(|err| {
        if let Some(database_err) = err.into_database_error() && database_err.is_unique_violation() {
            StatusCode::CONFLICT.into_response()
        } else {
            StatusCode::INTERNAL_SERVER_ERROR.into_response()}
    })?;
    Ok((StatusCode::CREATED, Json(user)).into_response())
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
#[allow(clippy::result_large_err)]
pub(crate) async fn login(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Basic>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Response, Response> {
    struct UserRow {
        id: Uuid,
        password_hash: String,
    }

    let email = Email::new(authorization.username().to_string())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Malformed email").into_response())?;
    let password = Password::new(authorization.password().to_string())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Malformed password").into_response())?;

    let user = sqlx::query_as!(
        UserRow,
        r#"
        SELECT id, password_hash
        FROM users
        WHERE email = $1
        "#,
        email.get(),
    )
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
    .ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())?;

    let password_hash = user.password_hash.clone();

    let valid = tokio::task::spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&password_hash).map_err(|_| ())?;

        Ok(Argon2::default()
            .verify_password(password.get().as_bytes(), &parsed_hash)
            .is_ok())
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?
    .map_err(|()| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
    if !valid {
        return Err(StatusCode::UNAUTHORIZED.into_response());
    }

    let (session, tokens) = Session::create(
        &state.pool,
        user.id,
        payload.title,
        SessionParams::default(),
    )
    .await
    .map_err(|err| match err {
        SessionError::ExpiredOrRevoked => StatusCode::UNAUTHORIZED.into_response(),
        err => {
            dbg!(&err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    })?;
    let auth_response = AuthResponse {
        access_token: BASE64_STANDARD.encode(tokens.access),
        refresh_token: BASE64_STANDARD.encode(tokens.refresh),
        expires_in: (session.refresh_expires_at.signed_duration_since(Utc::now()))
            .num_seconds()
            .cast_unsigned(),
    };

    Ok((StatusCode::OK, Json(auth_response)).into_response())
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
#[allow(clippy::result_large_err)]
pub(crate) async fn refresh(
    State(app_state): State<AppState>,
    Query(query): Query<RefreshQuery>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> Result<Response, Response> {
    let (session, tokens) = Session::refresh(
        &app_state.pool,
        query.user_id,
        String::from_utf8(
            BASE64_STANDARD
                .decode(authorization.token())
                .map_err(|_| StatusCode::BAD_REQUEST.into_response())?,
        )
        .map_err(|_| StatusCode::BAD_REQUEST.into_response())?,
    )
    .await
    .map_err(|err| match err {
        SessionError::ExpiredOrRevoked => StatusCode::UNAUTHORIZED.into_response(),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    })?;
    let auth_response = AuthResponse {
        access_token: BASE64_STANDARD.encode(tokens.access),
        refresh_token: BASE64_STANDARD.encode(tokens.refresh),
        expires_in: (session.refresh_expires_at.signed_duration_since(Utc::now()))
            .num_seconds()
            .cast_unsigned(),
    };

    Ok((StatusCode::OK, Json(auth_response)).into_response())
}
