use axum::{extract::Host, http::Uri, middleware, routing::get, Router};

use crate::{app_state::AppState, auth, ws};

pub const PATH_WS: &str = "/ws";
pub const PATH_AUTH: &str = "/auth";

pub fn routes(app_state: AppState) -> Router {
    let ws = Router::new()
        .route(PATH_WS, get(ws::ws_handler))
        .layer(middleware::from_fn_with_state(app_state.clone(), auth::ws_auth));
    let restricted = Router::new()
        .route("/rs", get(restricted));
    // let accessible = Router::new()
        // .route("auth", )
    Router::new()
        .nest("", ws)
        .nest("", restricted)
        .with_state(app_state)
}


async fn restricted(claims: auth::Claims) -> Result<String, auth::AuthError> {
    Ok(format!(
        "Welcome to the protected area :)\nYour data:\n{claims}",
    ))
}