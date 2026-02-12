use crate::Tool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, tool: Arc<dyn Tool>) {
        let mut tools = self.tools.write().await;
        tools.insert(tool.name().to_string(), tool);
    }

    pub async fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }

    pub async fn list_definitions(&self) -> Vec<serde_json::Value> {
        let tools = self.tools.read().await;
        tools.values().map(|t| {
            serde_json::json!({
                "name": t.name(),
                "description": t.description(),
                "parameters": t.parameters()
            })
        }).collect()
    }

    /// Return tool definitions filtered by an allowed-tools list.
    /// If `allowed_tools` is empty, returns ALL tools (backward compatible).
    pub async fn list_definitions_for_permissions(
        &self,
        allowed_tools: &[String],
    ) -> Vec<serde_json::Value> {
        if allowed_tools.is_empty() {
            return self.list_definitions().await;
        }
        let tools = self.tools.read().await;
        tools.values()
            .filter(|t| allowed_tools.iter().any(|a| a == t.name()))
            .map(|t| {
                serde_json::json!({
                    "name": t.name(),
                    "description": t.description(),
                    "parameters": t.parameters()
                })
            })
            .collect()
    }

    /// Check if a tool name is in the allowed list.
    /// If `allowed_tools` is empty, all tools are allowed (backward compatible).
    pub fn is_tool_allowed(tool_name: &str, allowed_tools: &[String]) -> bool {
        allowed_tools.is_empty() || allowed_tools.iter().any(|a| a == tool_name)
    }
}
