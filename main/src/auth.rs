use serde::{Deserialize, Serialize};
use axum::{
    async_trait, extract::{
        FromRef, FromRequestParts, Host, Query, Request, State
    }, http::request::Parts, middleware::Next, response::{IntoResponse, Redirect, Response}, Error, RequestPartsExt
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use std::fmt::Display;

use crate::app_state::AppState;
use crate::route;

#[derive(Deserialize)]
struct AuthQuery {
    t: String
}

pub async fn ws_auth(state: State<AppState>, request: Request, next: Next) 
    -> Result<impl IntoResponse, Response> {
    
    let token: Query<AuthQuery> = Query::try_from_uri(request.uri()).unwrap();

    Ok(next.run(request).await)
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
            .map_err(|_| AuthError("no".to_owned()))?;

        let redirect_uri = st.auth_service()
            .get_signin_url("http://".to_owned() + &hostname + route::PATH_AUTH);
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError(redirect_uri.clone()))?;
        
        let user = st.auth_service()
            .parse_jwt_token(bearer.token().to_owned())
            .map_err(|_| AuthError(redirect_uri))?;
        Ok(Claims{
            id: user.id
        })
    }
}

#[derive(Debug)]
pub struct AuthError(String);

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        Redirect::to(&self.0).into_response()
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