//! Bearer-auth extractor used by every authenticated handler.

use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use uuid::Uuid;

use crate::http::error::ApiError;
use crate::http::AppState;
use crate::jwt::TokenType;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub role: String,
}

impl AuthUser {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| ApiError::unauthorized("missing Authorization header"))?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| ApiError::unauthorized("invalid Authorization header format"))?;

        let parsed = state
            .jwt
            .parse_token(token)
            .map_err(|_| ApiError::unauthorized("invalid token"))?;

        if parsed.kind != TokenType::Access {
            return Err(ApiError::unauthorized("invalid token type"));
        }

        Ok(AuthUser {
            user_id: parsed.user_id,
            role: parsed.role,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AdminAuth(pub AuthUser);

impl FromRequestParts<Arc<AppState>> for AdminAuth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let user = <AuthUser as FromRequestParts<Arc<AppState>>>::from_request_parts(parts, state)
            .await?;
        if !user.is_admin() {
            return Err(ApiError::new(
                StatusCode::FORBIDDEN,
                crate::error::ErrorCode::Forbidden,
                "admin role required",
            ));
        }
        Ok(AdminAuth(user))
    }
}
