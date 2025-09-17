use std::{env, fs};

use backend_lib::connection::Connection;
use backend_lib::state::server_state::ServerState;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_module("backend_bin", log::LevelFilter::Trace)
        .filter_module("backend_lib", log::LevelFilter::Trace)
        .init();
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let listener = TcpListener::bind(&addr).await.unwrap();
    log::trace!("Listening on: {}", addr);
    let state = ServerState::new();
    while let Ok((stream, new_addr)) = listener.accept().await {
        let ws_stream = match tokio_tungstenite::accept_async(stream).await {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to accept connection!: {e}");
                continue;
            }
        };
        log::trace!("Connection accepted from: {}", new_addr);
        let connection = Connection::new(new_addr, ws_stream, state.clone());
        tokio::spawn(connection.handle_connection());
    }
}
