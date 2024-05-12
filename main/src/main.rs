use app_state::AppState;
use sqlx::{self, postgres::PgPoolOptions};
use std::{net::SocketAddr, time::Duration};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod client;
mod auth;
mod ws;
mod config;
mod app_state;
mod route;
mod world;

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
        tracing::warn!("Cannot load .env file")
    }
    
    let state = create_state().await;
    let app = route::routes(state);

    let host = config::load_env_var(config::HOST, "127.0.0.1");
    let port = config::load_env_var(config::PORT, "3000");
    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

async fn create_state() -> AppState {
    let db_url = config::load_env_var_or_fail(config::DATABASE_URL);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_url)
        .await
        .inspect_err(|err| {
            panic!("Cannot connect to database: {}", err);
        })
        .unwrap();
    if let Err(err) = sqlx::migrate!().run(&pool).await {
        panic!("Cannot run migrations: {}", err);
    }

    AppState::new(pool)
}