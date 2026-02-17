use serde::Deserialize;
use std::path::PathBuf;
use regex::Regex;
use std::fs;
use tracing::warn;
use directories::UserDirs;
use std::process::Command;
use std::time::{Duration, SystemTime};

const OPEN_SKILLS_REPO_URL: &str = "https://github.com/openai/skills.git";
const OPEN_SKILLS_SYNC_MARKER: &str = ".phoneclaw-open-skills-sync";
const OPEN_SKILLS_SYNC_INTERVAL_SECS: u64 = 60 * 60 * 24 * 7;

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
    pub version: Option<String>,
    pub location: Option<PathBuf>,
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

/// The TOML manifest structure for skill.toml
#[derive(Debug, Clone, Deserialize)]
pub struct SkillManifest {
    pub metadata: ManifestMetadata,
    pub permissions: Option<SkillPermissions>,
    pub requirements: Option<SkillRequirements>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ManifestMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub always: bool,
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
        let mut skills = Vec::new();

        if let Some(open_skills_dir) = ensure_open_skills_repo() {
            skills.extend(load_open_skills(&open_skills_dir));
        }

        let skills_dir = self.workspace_path.join("skills");
        if !skills_dir.exists() {
            return skills;
        }

        // We only want to look at immediate subdirectories of "skills"
        let entries = match fs::read_dir(&skills_dir) {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to read skills directory: {}", e);
                return skills;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            // Check for manifest first.
            let manifest_path = path.join("skill.toml");
            let manifest_path_upper = path.join("SKILL.toml");
            if manifest_path.exists() {
                match self.load_skill_from_manifest(&manifest_path) {
                    Ok(skill) => {
                        skills.push(skill);
                        continue;
                    }
                    Err(e) => {
                        warn!("Failed to load skill manifest at {:?}: {}", manifest_path, e);
                        // Fallthrough to try SKILL.md? No, simpler to prioritize manifest if present.
                    }
                }
            } else if manifest_path_upper.exists() {
                match self.load_skill_from_manifest(&manifest_path_upper) {
                    Ok(skill) => {
                        skills.push(skill);
                        continue;
                    }
                    Err(e) => {
                        warn!("Failed to load skill manifest at {:?}: {}", manifest_path_upper, e);
                    }
                }
            }

            // Fallback to SKILL.md (Legacy)
            let skill_md_path = path.join("SKILL.md");
            if skill_md_path.exists() {
                match self.load_skill_md(&skill_md_path) {
                    Ok(skill) => skills.push(skill),
                    Err(e) => warn!("Failed to load SKILL.md at {:?}: {}", skill_md_path, e),
                }
            }
        }

        skills
    }

    /// Load skill from skill.toml + README.md (or just description)
    fn load_skill_from_manifest(&self, path: &PathBuf) -> anyhow::Result<Skill> {
        let content = fs::read_to_string(path)?;
        let manifest: SkillManifest = toml::from_str(&content)?;

        // Attempt to load content from README.md in the same dir
        let readme_path = path.parent().unwrap().join("README.md");
        let body = if readme_path.exists() {
            fs::read_to_string(readme_path).unwrap_or_else(|_| manifest.metadata.description.clone())
        } else {
            manifest.metadata.description.clone()
        };

        let (available, missing) = self.check_requirements(&manifest.requirements);

        Ok(Skill {
            name: manifest.metadata.name,
            description: manifest.metadata.description,
            content: body,
            requirements: manifest.requirements,
            permissions: manifest.permissions,
            always: manifest.metadata.always,
            available,
            missing_requirements: missing,
            version: Some(manifest.metadata.version),
            location: Some(path.clone()),
        })
    }

    /// Load legacy SKILL.md format
    fn load_skill_md(&self, path: &PathBuf) -> anyhow::Result<Skill> {
        let content = fs::read_to_string(path)?;
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
            version: None,
            location: Some(path.clone()),
        })
    }

    fn extract_frontmatter<'a>(&self, content: &'a str) -> (String, &'a str) {
        let re = Regex::new(r"(?s)^---\n(.*?)\n---\n(.*)").unwrap();
        if let Some(caps) = re.captures(content) {
            let frontmatter = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let body = caps.get(2).map(|m| m.as_str()).unwrap_or(content);
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

fn resolve_open_skills_dir() -> Option<PathBuf> {
    UserDirs::new().map(|dirs| dirs.home_dir().join("open-skills"))
}

fn ensure_open_skills_repo() -> Option<PathBuf> {
    let repo_dir = resolve_open_skills_dir()?;

    if !repo_dir.exists() {
        if !clone_open_skills_repo(&repo_dir) {
            return None;
        }
        let _ = mark_open_skills_synced(&repo_dir);
        return Some(repo_dir);
    }

    if should_sync_open_skills(&repo_dir) {
        if pull_open_skills_repo(&repo_dir) {
            let _ = mark_open_skills_synced(&repo_dir);
        } else {
            warn!(
                "open-skills update failed; using local copy from {}",
                repo_dir.display()
            );
        }
    }

    Some(repo_dir)
}

fn load_open_skills(repo_dir: &PathBuf) -> Vec<Skill> {
    let mut skills = Vec::new();
    let Ok(entries) = fs::read_dir(repo_dir) else {
        return skills;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let is_md = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"));
        if !is_md {
            continue;
        }
        let is_readme = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("README.md"));
        if is_readme {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            let name = path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("open-skill")
                .to_string();
            skills.push(Skill {
                name,
                description: extract_description(&content),
                content,
                requirements: None,
                permissions: None,
                always: false,
                available: true,
                missing_requirements: Vec::new(),
                version: Some("open-skills".to_string()),
                location: Some(path),
            });
        }
    }

    skills
}

fn clone_open_skills_repo(repo_dir: &PathBuf) -> bool {
    if let Some(parent) = repo_dir.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            warn!(
                "failed to create open-skills parent directory {}: {}",
                parent.display(),
                err
            );
            return false;
        }
    }

    let output = Command::new("git")
        .args(["clone", "--depth", "1", OPEN_SKILLS_REPO_URL])
        .arg(repo_dir)
        .output();

    match output {
        Ok(result) if result.status.success() => true,
        Ok(result) => {
            warn!(
                "failed to clone open-skills: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            false
        }
        Err(err) => {
            warn!("failed to run git clone for open-skills: {}", err);
            false
        }
    }
}

fn pull_open_skills_repo(repo_dir: &PathBuf) -> bool {
    if !repo_dir.join(".git").exists() {
        return true;
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(repo_dir)
        .args(["pull", "--ff-only"])
        .output();

    match output {
        Ok(result) if result.status.success() => true,
        Ok(result) => {
            warn!(
                "failed to pull open-skills updates: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            false
        }
        Err(err) => {
            warn!("failed to run git pull for open-skills: {}", err);
            false
        }
    }
}

fn should_sync_open_skills(repo_dir: &PathBuf) -> bool {
    let marker = repo_dir.join(OPEN_SKILLS_SYNC_MARKER);
    let Ok(metadata) = fs::metadata(marker) else {
        return true;
    };
    let Ok(modified_at) = metadata.modified() else {
        return true;
    };
    let Ok(age) = SystemTime::now().duration_since(modified_at) else {
        return true;
    };
    age >= Duration::from_secs(OPEN_SKILLS_SYNC_INTERVAL_SECS)
}

fn mark_open_skills_synced(repo_dir: &PathBuf) -> anyhow::Result<()> {
    fs::write(repo_dir.join(OPEN_SKILLS_SYNC_MARKER), b"synced")?;
    Ok(())
}

fn extract_description(content: &str) -> String {
    content
        .lines()
        .find(|line| !line.starts_with('#') && !line.trim().is_empty())
        .unwrap_or("No description")
        .trim()
        .to_string()
}
