use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("AST parsing error: {0}")]
    AstParsingError(String),
    
    #[error("Invalid request format: {0}")]
    InvalidRequestFormat(String),
    
    #[error("Missing required field in request: {0}")]
    MissingField(String),
}
