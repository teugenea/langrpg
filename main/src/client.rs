use uuid::Uuid;

pub struct WsClient {
    id: Uuid
}

impl WsClient {

    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4()
        }
    }
}