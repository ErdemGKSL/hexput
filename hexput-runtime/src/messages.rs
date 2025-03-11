use hexput_ast_api::feature_flags::FeatureFlags;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebSocketRequest {
    pub id: String,
    pub action: String,
    pub code: String,
    #[serde(default)]
    pub options: AstParserOptions,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebSocketResponse {
    pub id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AstParserOptions {
    pub minify: bool,
    pub include_source_mapping: bool,
    pub no_object_constructions: bool,
    pub no_array_constructions: bool,
    pub no_object_navigation: bool,
    pub no_variable_declaration: bool,
    pub no_loops: bool,
    pub no_object_keys: bool,
    pub no_callbacks: bool,
    pub no_conditionals: bool,
    pub no_return_statements: bool,
    pub no_loop_control: bool,
    pub no_operators: bool,
    pub no_equality: bool,
    pub no_assignments: bool,
}

impl AstParserOptions {
    pub fn to_feature_flags(&self) -> FeatureFlags {
        FeatureFlags {
            allow_object_constructions: !self.no_object_constructions,
            allow_array_constructions: !self.no_array_constructions,
            allow_object_navigation: !self.no_object_navigation,
            allow_variable_declaration: !self.no_variable_declaration,
            allow_loops: !self.no_loops,
            allow_object_keys: !self.no_object_keys,
            allow_callbacks: !self.no_callbacks,
            allow_conditionals: !self.no_conditionals,
            allow_return_statements: !self.no_return_statements,
            allow_loop_control: !self.no_loop_control,
            allow_assignments: !self.no_assignments,
        }
    }
}
