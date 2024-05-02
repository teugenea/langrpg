use serde::{Deserialize, Serialize};
use axum::{
    async_trait, extract::{
        FromRef, FromRequestParts, Query, Request
    }, 
    http::{request::Parts, StatusCode}, middleware::Next, response::{IntoResponse, Response}, 
    Json, RequestPartsExt
};
use casdoor_rust_sdk::{AuthService, CasdoorConfig, CasdoorUser};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde_json::json;
use once_cell::sync::Lazy;
use tower_http::trace::TraceLayer;
use std::{borrow::Borrow, env::{self, VarError}};
use std::fmt::Display;

static CONF: Lazy<CasdoorConfig> = Lazy::new(|| {
    let url = env::var("CASDOOR_URL")
        .unwrap_or_else(|_| "http://localhost:8000".to_owned());
    tracing::debug!("test");
    CasdoorConfig::new(url, 
        "".to_owned(), "".to_owned(), 
        "".to_owned(), "".to_owned(), Some("".to_owned()))
});

#[derive(Deserialize)]
struct AuthQuery {
    t: String
}

pub async fn ws_auth(request: Request, next: Next) -> Result<impl IntoResponse, Response> {
    let token: Query<AuthQuery> = Query::try_from_uri(request.uri()).unwrap();

    Ok(next.run(request).await)
}

pub struct AuthState<'a> {
    service: AuthService<'a>
}

impl<'a> Clone for AuthState<'a> {
    fn clone(&self) -> Self {
        Self { service: AuthService::new(&CONF) }
    }
}

impl<'a> AuthState<'a> {
    pub fn new() -> Self {
        Self {
            service: AuthService::new(&CONF)
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        
        Ok(Claims{
            id: "".to_owned()
        })
    }
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    id: String
}

impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {}\n", self.id)
    }
}