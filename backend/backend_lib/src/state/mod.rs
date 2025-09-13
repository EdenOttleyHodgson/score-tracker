pub mod error;
pub mod room;
pub mod server_state;
pub mod session_data;
pub use room::Room;
pub use server_state::ServerState;
pub use session_data::SessionData;

pub type ID = usize;
