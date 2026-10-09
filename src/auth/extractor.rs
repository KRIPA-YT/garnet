use axum::{
    extract::FromRequestParts,
    http::{StatusCode, header::AUTHORIZATION, request::Parts},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{app::AppState, auth::token::Token};

pub(crate) struct AuthenticatedUser(pub Uuid);

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts.headers.get(AUTHORIZATION).ok_or_else(|| {
            (StatusCode::BAD_REQUEST, "`Authorization` header is missing").into_response()
        })?;
        let token = header
            .to_str()
            .ok()
            .and_then(|v| v.strip_prefix("Bearer "))
            .filter(|token| !token.is_empty())
            .and_then(Token::from_base64)
            .ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())?;
        let user_id = state
            .auth
            .authenticate_access(&token)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED.into_response())?;
        Ok(Self(user_id))
    }
}
