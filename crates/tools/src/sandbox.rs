use std::path::{Path, PathBuf};
use crate::ToolError;

/// Central sandbox configuration for all tools.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// All file operations are confined to this directory.
    pub workspace_path: PathBuf,
    /// Maximum execution time for shell commands (seconds).
    pub exec_timeout_secs: u64,
    /// Maximum combined stdout+stderr size (bytes).
    pub max_output_bytes: usize,
    /// Whether exec_cmd is allowed at all.
    pub exec_enabled: bool,
    /// Allowed domains for web_fetch / web_search. Empty = allow all.
    pub network_allowlist: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            workspace_path: PathBuf::from("workspace"),
            exec_timeout_secs: 30,
            max_output_bytes: 64 * 1024, // 64 KB
            exec_enabled: true,
            network_allowlist: Vec::new(),
        }
    }
}

/// Validate that a requested path is within the workspace boundary.
///
/// - Resolves relative paths against `workspace`.
/// - Canonicalizes to resolve symlinks and `..`.
/// - Rejects any path that escapes the workspace.
pub fn validate_path(workspace: &Path, requested: &str) -> Result<PathBuf, ToolError> {
    let requested_path = Path::new(requested);

    // Build absolute path
    let absolute = if requested_path.is_absolute() {
        requested_path.to_path_buf()
    } else {
        workspace.join(requested_path)
    };

    // Quick string-level check before canonicalize (catches obvious traversal)
    let as_str = absolute.to_string_lossy();
    if as_str.contains("..") {
        return Err(ToolError::ExecutionError(
            "Path traversal ('..') is not allowed".to_string(),
        ));
    }

    // For paths that don't exist yet (e.g. write_file creating new file),
    // canonicalize the parent directory instead.
    let resolved = if absolute.exists() {
        absolute.canonicalize().map_err(|e| {
            ToolError::ExecutionError(format!("Failed to resolve path: {}", e))
        })?
    } else {
        // Parent must exist for us to validate
        let parent = absolute.parent().ok_or_else(|| {
            ToolError::ExecutionError("Invalid path: no parent directory".to_string())
        })?;

        if !parent.exists() {
            // Create parent dirs within workspace
            let parent_resolved = if parent.is_absolute() {
                parent.to_path_buf()
            } else {
                workspace.join(parent)
            };
            // Verify parent is within workspace by string prefix (since it doesn't exist yet)
            let workspace_str = workspace.to_string_lossy();
            let parent_str = parent_resolved.to_string_lossy();
            if !parent_str.starts_with(workspace_str.as_ref()) {
                return Err(ToolError::ExecutionError(format!(
                    "Access denied: path '{}' is outside workspace '{}'",
                    requested, workspace.display()
                )));
            }
            return Ok(absolute);
        }

        let parent_canonical = parent.canonicalize().map_err(|e| {
            ToolError::ExecutionError(format!("Failed to resolve parent: {}", e))
        })?;

        let file_name = absolute.file_name().ok_or_else(|| {
            ToolError::ExecutionError("Invalid path: no file name".to_string())
        })?;

        parent_canonical.join(file_name)
    };

    // Canonicalize workspace for comparison
    let workspace_canonical = workspace.canonicalize().map_err(|e| {
        ToolError::ExecutionError(format!("Failed to resolve workspace: {}", e))
    })?;

    if !resolved.starts_with(&workspace_canonical) {
        return Err(ToolError::ExecutionError(format!(
            "Access denied: path '{}' is outside workspace '{}'",
            requested, workspace.display()
        )));
    }

    Ok(resolved)
}

/// Truncate a string to max_bytes, appending a notice if truncated.
pub fn truncate_output(output: &str, max_bytes: usize) -> String {
    if output.len() <= max_bytes {
        output.to_string()
    } else {
        let truncated = &output[..max_bytes];
        format!("{}\n\n--- OUTPUT TRUNCATED ({}B limit) ---", truncated, max_bytes)
    }
}
