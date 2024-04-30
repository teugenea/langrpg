use serde::Deserialize;
use axum::{
    extract::{
        Query, Request

    }, 
    middleware::Next, 
    response::{IntoResponse, Response},
};



#[derive(Deserialize)]
struct AuthQuery {
    t: String
}

pub async fn ws_auth(request: Request, next: Next) -> Result<impl IntoResponse, Response> {
    let token: Query<AuthQuery> = Query::try_from_uri(request.uri()).unwrap();

    Ok(next.run(request).await)
}