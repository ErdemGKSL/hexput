use crate::error::RuntimeError;
use crate::messages::{WebSocketRequest, WebSocketResponse};
use serde_json::Value;
use tracing::{debug, error, info};

pub async fn handle_request(request_data: &str) -> Result<String, RuntimeError> {
    let request: WebSocketRequest = serde_json::from_str(request_data)
        .map_err(|e| RuntimeError::InvalidRequestFormat(e.to_string()))?;

    debug!("Received request with ID: {}", request.id);
    debug!("Action: {}", request.action);

    match request.action.as_str() {
        "parse" => handle_parse_request(request).await,
        _ => {
            let response = WebSocketResponse {
                id: request.id,
                success: false,
                result: None,
                error: Some(format!("Unknown action: {}", request.action)),
            };
            Ok(serde_json::to_string(&response)?)
        }
    }
}

async fn handle_parse_request(request: WebSocketRequest) -> Result<String, RuntimeError> {
    let code = &request.code;
    let options = &request.options;

    let feature_flags = options.to_feature_flags();

    match hexput_ast_api::process_code(code, feature_flags) {
        Ok(program) => {
            let result = if options.minify {
                hexput_ast_api::to_json_string(&program, options.include_source_mapping)
            } else {
                hexput_ast_api::to_json_string_pretty(&program, options.include_source_mapping)
            };

            match result {
                Ok(json_str) => {
                    let value: Value = serde_json::from_str(&json_str)
                        .map_err(|e| RuntimeError::SerializationError(e))?;

                    let response = WebSocketResponse {
                        id: request.id.clone(),
                        success: true,
                        result: Some(value),
                        error: None,
                    };

                    info!("Successfully parsed AST for request: {}", request.id);
                    Ok(serde_json::to_string(&response)?)
                }
                Err(e) => {
                    error!("Serialization error: {}", e);
                    let response = WebSocketResponse {
                        id: request.id,
                        success: false,
                        result: None,
                        error: Some(format!("Error serializing AST: {}", e)),
                    };
                    Ok(serde_json::to_string(&response)?)
                }
            }
        }
        Err(e) => {
            error!("AST parsing error: {}", e);
            let response = WebSocketResponse {
                id: request.id,
                success: false,
                result: None,
                error: Some(format!("Error parsing AST: {}", e)),
            };
            Ok(serde_json::to_string(&response)?)
        }
    }
}
