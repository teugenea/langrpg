use casdoor_rust_sdk::AuthService;
use sqlx::{Pool, Postgres};
use tokio::sync::{Mutex, MutexGuard, RwLock};
use crate::{client::WsClient, config};
use std::{collections::HashMap, sync::Arc};

pub struct AppState {
    auth_service: AuthService<'static>,
    db_pool: Pool<Postgres>,
    clients: Arc<ClientsState>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self { 
            auth_service: AuthService::new(&config::CASDOOR_CONF),
            db_pool: self.db_pool.clone(),
            clients: self.clients.clone(),
        }
    }
}

impl AppState {
    pub fn new(db_pool: Pool<Postgres>) -> Self {
        Self {
            auth_service: AuthService::new(&config::CASDOOR_CONF),
            db_pool,
            clients: Arc::new(ClientsState::new()),
        }
    }

    pub fn auth_service(&self) -> &AuthService {
        &self.auth_service
    }

    pub fn db_pool(&self) -> &Pool<Postgres> {
        &self.db_pool
    }

    pub fn clients(&self) -> Arc<ClientsState> {
        self.clients.clone()
    }
    
}

#[derive(Clone)]
pub struct ClientHandle(pub Arc<Mutex<WsClient>>);

impl ClientHandle {
    pub async fn lock(&self) -> MutexGuard<'_, WsClient> {
        self.0.lock().await
    }
}

pub struct ClientsState {
    clients: RwLock<HashMap<String, ClientHandle>>,
}

impl ClientsState {

    pub fn new() -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_client(&self, id: &str) -> Option<ClientHandle> {
        let guard = &self.clients.read().await;
        guard.get(id).cloned()
    }

    pub async fn insert_client(&self, client: WsClient) -> ClientHandle {
        let guard =  &mut self.clients.write().await;
        let id = client.id().to_string();
        let cl = ClientHandle(Arc::new(Mutex::new(client)));
        guard.insert(id, cl.clone());
        return cl;
    }

    pub async fn del_client(&self, id: &str) -> Option<ClientHandle> {
        let guard =  &mut self.clients.write().await;
        guard.remove(id)
    }
}