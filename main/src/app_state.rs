use casdoor_rust_sdk::AuthService;
use sqlx::{Pool, Postgres};
use crate::config;

pub struct AppState {
    auth_service: AuthService<'static>,
    db_pool: Pool<Postgres>
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self { 
            auth_service: AuthService::new(&config::CASDOOR_CONF),
            db_pool: self.db_pool.clone(),
        }
    }
}

impl AppState {
    pub fn new(db_pool: Pool<Postgres>) -> Self {
        Self {
            auth_service: AuthService::new(&config::CASDOOR_CONF),
            db_pool
        }
    }

    pub fn auth_service(&self) -> &AuthService {
        &self.auth_service
    }

    pub fn db_pool(&self) -> &Pool<Postgres> {
        &self.db_pool
    }
}