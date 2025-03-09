use crate::error::RuntimeError;
use crate::handler::handle_request;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

pub struct ServerConfig {
    pub address: String,
}

pub async fn run_server(config: ServerConfig) -> Result<(), RuntimeError> {
    let addr = config.address.parse::<SocketAddr>().map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid server address")
    })?;

        let listener = TcpListener::bind(&addr).await?;
    info!("WebSocket server listening on: {}", addr);

        let active_connections = Arc::new(Mutex::new(0));

        while let Ok((stream, peer_addr)) = listener.accept().await {
        info!("New connection from: {}", peer_addr);
        
                let connections = active_connections.clone();
        
                {
            let mut count = connections.lock().await;
            *count += 1;
            info!("Active connections: {}", *count);
        }

                tokio::spawn(async move {
            match handle_connection(stream, peer_addr).await {
                Ok(_) => info!("Connection from {} closed gracefully", peer_addr),
                Err(e) => error!("Error handling connection from {}: {}", peer_addr, e),
            }
            
                        let mut count = connections.lock().await;
            *count -= 1;
            info!("Connection closed. Active connections: {}", *count);
        });
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream, peer_addr: SocketAddr) -> Result<(), RuntimeError> {
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    info!("WebSocket connection established with: {}", peer_addr);

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!("Received text message from {}", peer_addr);
                
                                let response = match handle_request(&text).await {
                    Ok(resp) => Message::Text(resp),
                    Err(e) => {
                        error!("Error processing request: {}", e);
                        Message::Text(format!("{{\"error\":\"Internal server error: {}\"}}",
                            e.to_string().replace('"', "\\\"")))
                    }
                };
                
                                if let Err(e) = ws_sender.send(response).await {
                    error!("Error sending response to {}: {}", peer_addr, e);
                }
            },
            Ok(Message::Ping(data)) => {
                                if let Err(e) = ws_sender.send(Message::Pong(data)).await {
                    error!("Error sending pong to {}: {}", peer_addr, e);
                }
            },
            Ok(Message::Close(_)) => {
                info!("Received close message from {}", peer_addr);
                break;
            },
            Err(e) => {
                error!("Error reading message from {}: {}", peer_addr, e);
                break;
            },
            _ => {}         }
    }

        info!("Closing connection with: {}", peer_addr);
    Ok(())
}
