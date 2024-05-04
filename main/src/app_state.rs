use casdoor_rust_sdk::AuthService;
use crate::config;

pub struct AppState {
    auth_service: AuthService<'static>
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self { 
            auth_service: AuthService::new(&config::CASDOOR_CONF),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            auth_service: AuthService::new(&config::CASDOOR_CONF)
        }
    }

    pub fn auth_service(&self) -> &AuthService {
        &self.auth_service
    }
}