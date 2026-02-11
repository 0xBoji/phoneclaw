use crate::{Tool, ToolError};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::process::Command;

pub struct ExecTool {
    workspace: String,
}

impl ExecTool {
    pub fn new(workspace: String) -> Self {
        Self { workspace }
    }
}

#[derive(Deserialize)]
struct ExecArgs {
    command: String,
}

#[async_trait]
impl Tool for ExecTool {
    fn name(&self) -> &str {
        "exec_cmd"
    }

    fn description(&self) -> &str {
        "Execute a shell command in the workspace."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The shell command to execute."
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let args: ExecArgs = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArgs(e.to_string()))?;

        // Basic security check: prevent cd ..
        if args.command.contains("..") {
            return Err(ToolError::ExecutionError("Command contains disallowed '..' sequence".to_string()));
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(&args.command)
            .current_dir(&self.workspace)
            .output()
            .await
            .map_err(|e: std::io::Error| ToolError::ExecutionError(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = String::new();
        if !stdout.is_empty() {
            result.push_str(&format!("STDOUT:\n{}\n", stdout));
        }
        if !stderr.is_empty() {
            result.push_str(&format!("STDERR:\n{}\n", stderr));
        }

        if result.is_empty() {
             Ok("(no output)".to_string())
        } else {
             Ok(result)
        }
    }
}
