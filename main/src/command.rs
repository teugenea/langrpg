pub enum Command {
    Hit {target_id: u32},
    Move {x: u32, y: u32},
}