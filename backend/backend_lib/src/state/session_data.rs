use super::room::RoomCode;

#[derive(Debug)]
pub struct SessionData {
    current_room: Option<RoomCode>,
}

impl SessionData {
    pub fn new() -> Self {
        Self { current_room: None }
    }

    pub fn current_room(&self) -> Option<RoomCode> {
        self.current_room
    }
    pub fn set_current_room(&mut self, code: RoomCode) {
        self.current_room = Some(code)
    }
}
