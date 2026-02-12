use crate::sandbox::{SandboxConfig, validate_path};
use crate::{Tool, ToolError};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::fs;

// ─── Write File ───

pub struct WriteFileTool {
    sandbox: SandboxConfig,
}

impl WriteFileTool {
    pub fn new(sandbox: SandboxConfig) -> Self {
        Self { sandbox }
    }
}

#[derive(Deserialize)]
struct WriteFileArgs {
    path: String,
    content: String,
}

#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file at the specified path (workspace only)."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to write (relative to workspace)."
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file."
                }
            },
            "required": ["path", "content"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let args: WriteFileArgs = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArgs(e.to_string()))?;

        let safe_path = validate_path(&self.sandbox.workspace_path, &args.path)?;

        // Create parent directories if needed
        if let Some(parent) = safe_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| ToolError::ExecutionError(format!("Failed to create dirs: {}", e)))?;
            }
        }

        fs::write(&safe_path, &args.content)
            .await
            .map_err(|e| ToolError::ExecutionError(e.to_string()))?;

        Ok(format!("Successfully wrote to {}", safe_path.display()))
    }
}

// ─── Read File ───

pub struct ReadFileTool {
    sandbox: SandboxConfig,
}

impl ReadFileTool {
    pub fn new(sandbox: SandboxConfig) -> Self {
        Self { sandbox }
    }
}

#[derive(Deserialize)]
struct ReadFileArgs {
    path: String,
}

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read content from a file at the specified path (workspace only)."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the file to read (relative to workspace)."
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let args: ReadFileArgs = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArgs(e.to_string()))?;

        let safe_path = validate_path(&self.sandbox.workspace_path, &args.path)?;

        let content = fs::read_to_string(&safe_path)
            .await
            .map_err(|e| ToolError::ExecutionError(e.to_string()))?;

        Ok(content)
    }
}

// ─── List Directory ───

pub struct ListDirTool {
    sandbox: SandboxConfig,
}

impl ListDirTool {
    pub fn new(sandbox: SandboxConfig) -> Self {
        Self { sandbox }
    }
}

#[derive(Deserialize)]
struct ListDirArgs {
    path: String,
}

#[async_trait]
impl Tool for ListDirTool {
    fn name(&self) -> &str {
        "list_dir"
    }

    fn description(&self) -> &str {
        "List files and directories in a given path (workspace only)."
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "The path to the directory to list (relative to workspace)."
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String, ToolError> {
        let args: ListDirArgs = serde_json::from_value(args)
            .map_err(|e| ToolError::InvalidArgs(e.to_string()))?;

        let safe_path = validate_path(&self.sandbox.workspace_path, &args.path)?;

        let mut entries = fs::read_dir(&safe_path)
            .await
            .map_err(|e| ToolError::ExecutionError(e.to_string()))?;

        let mut result = String::new();
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| ToolError::ExecutionError(e.to_string()))?
        {
            let file_type = entry
                .file_type()
                .await
                .map_err(|e| ToolError::ExecutionError(e.to_string()))?;
            let name = entry.file_name().to_string_lossy().to_string();
            let type_str = if file_type.is_dir() { "DIR" } else { "FILE" };
            result.push_str(&format!("[{}] {}\n", type_str, name));
        }

        if result.is_empty() {
            Ok("(empty directory)".to_string())
        } else {
            Ok(result)
        }
    }
}
