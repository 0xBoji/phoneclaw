use serde::Deserialize;
use std::path::PathBuf;
use walkdir::WalkDir;
use regex::Regex;
use std::fs;

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub content: String,
    pub requirements: Option<SkillRequirements>,
    pub permissions: Option<SkillPermissions>,
    pub always: bool,
    pub available: bool,
    pub missing_requirements: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub always: bool,
    pub requires: Option<SkillRequirements>,
    pub permissions: Option<SkillPermissions>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SkillRequirements {
    #[serde(default)]
    pub bins: Vec<String>,
    #[serde(default)]
    pub env: Vec<String>,
}

/// Permission manifest for a skill — declares what the skill is allowed to do.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SkillPermissions {
    /// Tool names this skill is allowed to use (e.g. ["exec_cmd", "read_file"]).
    #[serde(default)]
    pub tools: Vec<String>,
    /// File system scope: "workspace" (default) or "system".
    #[serde(default = "default_fs_scope")]
    pub fs_scope: String,
    /// Allowed network domains for web tools (e.g. ["api.github.com"]).
    #[serde(default)]
    pub network_domains: Vec<String>,
    /// Override max exec timeout for this skill (seconds).
    pub max_exec_timeout: Option<u64>,
}

fn default_fs_scope() -> String {
    "workspace".to_string()
}

pub struct SkillsLoader {
    workspace_path: PathBuf,
}

impl SkillsLoader {
    pub fn new(workspace_path: PathBuf) -> Self {
        Self { workspace_path }
    }

    pub fn list_skills(&self) -> Vec<Skill> {
        let skills_dir = self.workspace_path.join("skills");
        let mut skills = Vec::new();

        if !skills_dir.exists() {
            return skills;
        }

        for entry in WalkDir::new(skills_dir).min_depth(1).max_depth(2) {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if entry.file_name() == "SKILL.md" {
                if let Ok(skill) = self.load_skill(entry.path().to_path_buf()) {
                    skills.push(skill);
                }
            }
        }

        skills
    }

    fn load_skill(&self, path: PathBuf) -> anyhow::Result<Skill> {
        let content = fs::read_to_string(&path)?;
        let (frontmatter_str, body) = self.extract_frontmatter(&content);

        let metadata: SkillMetadata = serde_json::from_str(&frontmatter_str)
            .unwrap_or_else(|_| SkillMetadata {
                name: path.parent().unwrap().file_name().unwrap().to_string_lossy().to_string(),
                description: "No description provided".to_string(),
                always: false,
                requires: None,
                permissions: None,
            });

        let (available, missing) = self.check_requirements(&metadata.requires);

        Ok(Skill {
            name: metadata.name,
            description: metadata.description,
            content: body.to_string(),
            requirements: metadata.requires,
            permissions: metadata.permissions,
            always: metadata.always,
            available,
            missing_requirements: missing,
        })
    }

    fn extract_frontmatter<'a>(&self, content: &'a str) -> (String, &'a str) {
        let re = Regex::new(r"(?s)^---\n(.*?)\n---\n(.*)").unwrap();
        if let Some(caps) = re.captures(content) {
            let frontmatter = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let body = caps.get(2).map(|m| m.as_str()).unwrap_or(content);
            // YAML frontmatter might be JSON in the original Go code, 
            // but let's assume it's JSON for now based on the Go struct tags `json:...`.
            // Wait, the Go code uses `json.Unmarshal`. So it expects JSON in frontmatter!
             // That's unusual but I will stick to it.
            return (frontmatter.to_string(), body);
        }
        ("{}".to_string(), content)
    }

    fn check_requirements(&self, requires: &Option<SkillRequirements>) -> (bool, Vec<String>) {
        let mut missing = Vec::new();
        if let Some(req) = requires {
            for bin in &req.bins {
                 if which::which(bin).is_err() {
                     missing.push(format!("CLI: {}", bin));
                 }
            }
            for env in &req.env {
                if std::env::var(env).is_err() {
                    missing.push(format!("ENV: {}", env));
                }
            }
        }
        (missing.is_empty(), missing)
    }
}
