use jsonwebtoken::{decode, Algorithm, Validation};
use serde::{Deserialize, Serialize};
use axum::{
    async_trait, extract::{
        FromRef, FromRequestParts, Host, Query, Request, State
    }, http::{request::Parts, StatusCode}, 
    middleware::Next, response::{IntoResponse, Redirect, Response}, 
    RequestPartsExt
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use std::fmt::Display;

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

pub async fn ws_auth(state: State<AppState>, request: Request, next: Next) 
    -> Result<impl IntoResponse, Response> {
    
    let token: Query<WsAuthQuery> = Query::try_from_uri(request.uri())
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Cannot get token").into_response())?;
    parse_token(&token.0.t, "".to_owned())
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Cannot get token").into_response())?;

    Ok(next.run(request).await)
}

pub async fn auth_by_code(state: State<AppState>, query: Query<AuthQuery>) 
    -> Result<impl IntoResponse, AuthError> {
        
    let code = query.code.clone();
    let token = tokio::task::block_in_place(move || {
        state.auth_service()
            .get_auth_token(code)
            .map_err(|err| AuthError::ServerError(format!("Cannot get token by code: {}", err)))
    })?;
    
    Ok(token)
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
    AppState: FromRef<S>
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let st = AppState::from_ref(state);

        let Host(hostname) = parts
            .extract::<Host>()
            .await
            .map_err(|_| AuthError::ServerError("Cannot extract hostname".to_owned()))?;

        let redirect_uri = st.auth_service()
            .get_signin_url("http://".to_owned() + &hostname + route::PATH_AUTH);

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::WithRedirect(redirect_uri.clone()))?;
        
        parse_token(bearer.token(), redirect_uri)
    }
}

fn parse_token(token: &str, redirect_uri: String) -> Result<Claims, AuthError> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[config::load_env_var_or_fail(config::IAM_CLIENT_ID)]);
    let token_data = 
        decode::<Claims>(token, &config::JWT_DECODING_KEY, &validation)
        .map_err(|_| AuthError::WithRedirect(redirect_uri))?;

    Ok(token_data.claims)
}

#[derive(Debug)]
pub enum AuthError {
    WithRedirect(String),
    ServerError(String)
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            AuthError::WithRedirect(uri) => Redirect::to(&uri).into_response(),
            AuthError::ServerError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err).into_response()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    id: String,
    name: String,
    groups: Vec<String>
}

impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}