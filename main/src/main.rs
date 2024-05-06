use app_state::AppState;
use axum::{
    Router, routing::get,
    middleware::{self}
};
use std::net::SocketAddr;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod client;
mod auth;
mod ws;
mod config;
mod app_state;
mod route;

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
    
    let app_state = AppState::new();
    let app = route::routes(app_state);

    let host = config::load_env_var(config::HOST, "127.0.0.1");
    let port = config::load_env_var(config::PORT, "3000");
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}