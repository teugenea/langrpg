use uuid::Uuid;

pub struct WsClient {
    id: Uuid,
    x: i32
}

impl WsClient {

    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            x: 0,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn set_x(&mut self, x: i32) {
        self.x = x
    }
}