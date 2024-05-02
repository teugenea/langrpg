use auth::AuthState;
use axum::{
    Router, routing::get,
    middleware::{self},
    extract::State
};
use casdoor_rust_sdk::CasdoorConfig;
use std::net::SocketAddr;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod client;
mod auth;
mod ws;

#[tokio::main]
async fn main() {

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "main=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    if let Err(_) = dotenvy::dotenv() {

    }
    
    let ws = Router::new()
        .route("/ws", get(ws::ws_handler))
        .layer(middleware::from_fn(auth::ws_auth));
    let restricted = Router::new()
        .route("/rs", get(restricted))
        .with_state(AuthState::new());
    let app = Router::new()
        .nest("", ws)
        .nest("", restricted);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

async fn restricted(claims: auth::Claims) -> Result<String, auth::AuthError> {
    Ok(format!(
        "Welcome to the protected area :)\nYour data:\n{claims}",
    ))
}