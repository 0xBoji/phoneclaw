use crate::{Tool, ToolError};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

/// Trait to abstrat the Android JNI calls.
/// This allows us to mock it for testing or implement it in the mobile-jni crate.
#[async_trait]
pub trait AndroidBridge: Send + Sync {
    async fn click(&self, x: f32, y: f32) -> Result<bool, String>;
    async fn scroll(&self, x1: f32, y1: f32, x2: f32, y2: f32) -> Result<bool, String>;
    async fn back(&self) -> Result<bool, String>;
    async fn home(&self) -> Result<bool, String>;
    // Future: input_text, dump_hierarchy
}

pub struct AndroidActionTool {
    bridge: Arc<dyn AndroidBridge>,
}

impl AndroidActionTool {
    pub fn new(bridge: Arc<dyn AndroidBridge>) -> Self {
        Self { bridge }
    }
}

#[async_trait]
impl Tool for AndroidActionTool {
    fn name(&self) -> &str {
        "android_action"
    }

    fn description(&self) -> &str {
        "Perform actions on the Android device (click, scroll, back, home)."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["click", "scroll", "back", "home"],
                    "description": "The action to perform."
                },
                "x": { "type": "number", "description": "X coordinate (for click)" },
                "y": { "type": "number", "description": "Y coordinate (for click)" },
                "x1": { "type": "number", "description": "Start X (for scroll)" },
                "y1": { "type": "number", "description": "Start Y (for scroll)" },
                "x2": { "type": "number", "description": "End X (for scroll)" },
                "y2": { "type": "number", "description": "End Y (for scroll)" }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let action = args["action"].as_str().ok_or(ToolError::InvalidArgs("action required".into()))?;

        let result = match action {
            "click" => {
                let x = args["x"].as_f64().ok_or(ToolError::InvalidArgs("x required".into()))? as f32;
                let y = args["y"].as_f64().ok_or(ToolError::InvalidArgs("y required".into()))? as f32;
                self.bridge.click(x, y).await
            }
            "scroll" => {
                let x1 = args["x1"].as_f64().ok_or(ToolError::InvalidArgs("x1 required".into()))? as f32;
                let y1 = args["y1"].as_f64().ok_or(ToolError::InvalidArgs("y1 required".into()))? as f32;
                let x2 = args["x2"].as_f64().ok_or(ToolError::InvalidArgs("x2 required".into()))? as f32;
                let y2 = args["y2"].as_f64().ok_or(ToolError::InvalidArgs("y2 required".into()))? as f32;
                self.bridge.scroll(x1, y1, x2, y2).await
            }
            "back" => self.bridge.back().await,
            "home" => self.bridge.home().await,
            _ => return Err(ToolError::InvalidArgs(format!("Unknown action: {}", action))),
        };

        match result {
            Ok(true) => Ok(format!("Action '{}' performed successfully", action)),
            Ok(false) => Err(ToolError::ExecutionError(format!("Action '{}' failed (service returned false)", action))),
            Err(e) => Err(ToolError::ExecutionError(format!("Action '{}' error: {}", action, e))),
        }
    }
}
