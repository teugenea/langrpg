use std::time::Instant;

use uuid::Uuid;

#[derive(Copy, Clone)]
pub struct WsClient {
    pub id: Uuid,
    pub hb: Instant
}

impl WsClient {

    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            hb: Instant::now()
        }
    }
}