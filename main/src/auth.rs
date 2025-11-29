use headers::HeaderMapExt;
use jsonwebtoken::{decode, Algorithm, Validation};
use serde::{Deserialize, Serialize};
use axum::{
    extract::{
        FromRef, FromRequestParts, Query, Request, State
    }, http::{request::Parts, StatusCode},
    middleware::Next, response::{IntoResponse, Redirect, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use std::{error::Error, fmt::Display};

use crate::{app_state::AppState, config};
use crate::route;

#[derive(Deserialize)]
struct WsAuthQuery {
    t: String
}

#[derive(Deserialize)]
pub struct AuthQuery {
    code: String,
    state: String
}

pub async fn ws_auth(_state: State<AppState>, request: Request, next: Next) 
    -> Result<impl IntoResponse, Response> {
    
    let token: Query<WsAuthQuery> = Query::try_from_uri(request.uri())
        .map_err(|err| AuthError::from_err("Cannot extract token", Box::new(err), StatusCode::UNAUTHORIZED).into_response())?;
    
    let (mut parts, body) = request.into_parts();
    let auth_header = Authorization::bearer(&token.t)
        .map_err(|err| AuthError::from_err("Cannot extract token", Box::new(err), StatusCode::UNAUTHORIZED).into_response())?;
    parts.headers.typed_insert(auth_header);

    Ok(next.run(Request::from_parts(parts, body)).await)
}

pub async fn auth_by_code(state: State<AppState>, query: Query<AuthQuery>) 
    -> Result<impl IntoResponse, AuthError> {
        
    let code = query.code.clone();
    let token = tokio::task::block_in_place(move || {
        state.auth_service()
            .get_auth_token(code)
            .map_err(|err| AuthError::from_err("Cannot get token by code", err, StatusCode::UNAUTHORIZED))
    })?;
    
    Ok(token)
}

//#[async_trait::async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
    AppState: FromRef<S>
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let st = AppState::from_ref(state);

        let hostname = parts
            .headers
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("localhost");

        let redirect_uri = st
            .auth_service()
            .get_signin_url(format!("http://{}{}", hostname, route::PATH_AUTH));

        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|err| {
                    AuthError::from_err_redirect(
                        "Cannot extract token",
                        Box::new(err),
                        redirect_uri.clone(),
                    )
                })?;

        parse_token(bearer.token(), redirect_uri)
    }
}

fn parse_token(token: &str, redirect_uri: String) -> Result<Claims, AuthError> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[config::load_env_var_or_fail(config::IAM_CLIENT_ID)]);
    let token_data = 
        decode::<Claims>(token, &config::JWT_DECODING_KEY, &validation)
            .map_err(|err| AuthError::from_err_redirect("Cannot decode token", Box::new(err), redirect_uri))?;

    Ok(token_data.claims)
}

#[derive(Debug)]
pub struct  AuthError {
    err: Box<dyn Error>,
    status_code: StatusCode,
    message: String,
    redirect_uri: Option<String>
}

impl AuthError {

    fn from_err_redirect(message: &str, err: Box<dyn Error>, redirect_uri: String) -> Self {
        Self {
            err,
            status_code: StatusCode::PERMANENT_REDIRECT,
            message: message.to_owned(),
            redirect_uri: Some(redirect_uri)
        }
    }

    fn from_err(message: &str, err: Box<dyn Error>, status_code: StatusCode) -> Self {
        Self {
            err,
            status_code,
            message: message.to_owned(),
            redirect_uri: None
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let msg = format!("{}: {}", self.message, self.err);
        tracing::error!("{}", msg);
        if let Some(uri) = self.redirect_uri {
            return Redirect::to(&uri).into_response();
        }
        (self.status_code, msg).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub name: String,
    pub groups: Vec<String>
}

impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}