//! Export/import of the `cvm.yaml` manifest.
//!
//! Security guarantee: this module only ever reads or writes three things
//! inside an environment directory: `settings.json` (permissions +
//! `mcpServers`), the `skills/` subdirectory, and the *names* (never the
//! values) of variables in `.env`. It never opens files such as
//! `auth.json`, `.credentials.json`, or `history.jsonl` - those simply
//! never appear in the read/write paths below, so no filtering step can
//! accidentally miss them.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::env;

/// Filenames that must never be read during export or written during import.
/// Kept as an explicit denylist for documentation and tests; the real
/// guarantee comes from the fact that no function below opens anything
/// other than `settings.json` and `skills/`.
#[cfg(test)]
pub const SENSITIVE_FILES: &[&str] = &[
    "auth.json",
    "credentials.json",
    ".credentials.json",
    "history.json",
    "history.jsonl",
    "memory",
];

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Settings::is_empty")]
    pub settings: Settings,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub mcp_servers: BTreeMap<String, McpServer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,
    /// Names (never values) of variables expected in this environment's
    /// `.env` file - e.g. MCP server credentials. `cvm import` recreates
    /// `.env` with these names as blank placeholders for you to fill in.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env_vars: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_tools: Vec<String>,
}

impl Settings {
    fn is_empty(&self) -> bool {
        self.allowed_tools.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<BTreeMap<String, String>>,
}

/// The subset of Claude Code's `settings.json` that cvm understands.
/// Unknown keys are preserved via `extra` so import never destroys settings
/// cvm doesn't manage.
#[derive(Debug, Default, Serialize, Deserialize)]
struct ClaudeSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    permissions: Option<Permissions>,
    #[serde(
        default,
        rename = "mcpServers",
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    mcp_servers: BTreeMap<String, McpServer>,
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Permissions {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    allow: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    deny: Vec<String>,
}

fn read_claude_settings(env_dir: &Path) -> Result<ClaudeSettings> {
    let path = env_dir.join("settings.json");
    if !path.exists() {
        return Ok(ClaudeSettings::default());
    }
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

fn write_claude_settings(env_dir: &Path, settings: &ClaudeSettings) -> Result<()> {
    let path = env_dir.join("settings.json");
    let json = serde_json::to_string_pretty(settings)?;
    fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn list_skills(env_dir: &Path) -> Result<Vec<String>> {
    let skills_dir = env_dir.join("skills");
    if !skills_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut skills = Vec::new();
    for entry in fs::read_dir(&skills_dir)
        .with_context(|| format!("failed to read {}", skills_dir.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                skills.push(name.to_string());
            }
        }
    }
    skills.sort();
    Ok(skills)
}

/// Builds a shareable [`Manifest`] from an environment directory. Only reads
/// `settings.json`, `skills/`, and the *names* of variables in `.env` -
/// never auth, credentials, history, or `.env` values.
pub fn export_env(
    env_dir: &Path,
    name: &str,
    version: &str,
    description: Option<String>,
) -> Result<Manifest> {
    let claude_settings = read_claude_settings(env_dir)?;
    let allowed_tools = claude_settings
        .permissions
        .map(|p| p.allow)
        .unwrap_or_default();
    let skills = list_skills(env_dir)?;
    let mut env_vars: Vec<String> = env::load_env_file(env_dir)?
        .into_iter()
        .map(|(key, _)| key)
        .collect();
    env_vars.sort();

    Ok(Manifest {
        name: name.to_string(),
        version: version.to_string(),
        description,
        settings: Settings { allowed_tools },
        mcp_servers: claude_settings.mcp_servers,
        skills,
        env_vars,
    })
}

pub fn write_manifest(manifest: &Manifest, path: &Path) -> Result<()> {
    let yaml = serde_yaml::to_string(manifest)?;
    fs::write(path, yaml).with_context(|| format!("failed to write {}", path.display()))
}

pub fn read_manifest(path: &Path) -> Result<Manifest> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_yaml::from_str(&raw)
        .with_context(|| format!("failed to parse manifest {}", path.display()))
}

/// Applies a [`Manifest`] to an environment directory: merges permissions
/// and `mcpServers` into `settings.json`, creates placeholder skill
/// directories for any skill listed, and ensures `.env` has an entry (blank
/// if new, untouched if already set) for every variable name listed. Never
/// touches auth, credentials, or history files.
pub fn apply_manifest(manifest: &Manifest, env_dir: &Path) -> Result<()> {
    fs::create_dir_all(env_dir)
        .with_context(|| format!("failed to create {}", env_dir.display()))?;

    let mut claude_settings = read_claude_settings(env_dir)?;
    let mut permissions = claude_settings.permissions.take().unwrap_or_default();
    permissions.allow = manifest.settings.allowed_tools.clone();
    claude_settings.permissions = Some(permissions);
    claude_settings.mcp_servers = manifest.mcp_servers.clone();
    write_claude_settings(env_dir, &claude_settings)?;

    if !manifest.skills.is_empty() {
        let skills_dir = env_dir.join("skills");
        for skill in &manifest.skills {
            let skill_dir = skills_dir.join(skill);
            fs::create_dir_all(&skill_dir)
                .with_context(|| format!("failed to create {}", skill_dir.display()))?;
            let stub = skill_dir.join("SKILL.md");
            if !stub.exists() {
                fs::write(
                    &stub,
                    format!(
                        "# {skill}\n\n\
                         This skill was registered via `cvm import` but its content was not \
                         bundled in the manifest. Replace this stub with the actual skill \
                         definition, or reinstall it from its original source.\n"
                    ),
                )
                .with_context(|| format!("failed to write {}", stub.display()))?;
            }
        }
    }

    if !manifest.env_vars.is_empty() {
        let mut vars: BTreeMap<String, String> = env::load_env_file(env_dir)?.into_iter().collect();
        for name in &manifest.env_vars {
            vars.entry(name.clone()).or_default();
        }
        env::write_env_file(env_dir, &vars)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap as Map;

    #[test]
    fn export_never_reads_sensitive_files() {
        let dir = tempfile::tempdir().unwrap();
        for name in SENSITIVE_FILES {
            fs::write(dir.path().join(name), "super-secret").unwrap();
        }
        fs::write(
            dir.path().join("settings.json"),
            r#"{"permissions":{"allow":["read-file"]}}"#,
        )
        .unwrap();

        let manifest = export_env(dir.path(), "demo", "1.0.0", None).unwrap();
        let yaml = serde_yaml::to_string(&manifest).unwrap();

        assert!(!yaml.contains("super-secret"));
        assert_eq!(
            manifest.settings.allowed_tools,
            vec!["read-file".to_string()]
        );
    }

    #[test]
    fn round_trips_mcp_servers_through_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let mut servers = Map::new();
        servers.insert(
            "postgres".to_string(),
            McpServer {
                command: "npx".to_string(),
                args: vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-postgres".to_string(),
                ],
                env: None,
            },
        );
        let manifest = Manifest {
            name: "demo".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            settings: Settings::default(),
            mcp_servers: servers,
            skills: vec![],
            env_vars: vec![],
        };

        apply_manifest(&manifest, dir.path()).unwrap();
        let reexported = export_env(dir.path(), "demo", "1.0.0", None).unwrap();

        assert_eq!(reexported.mcp_servers.len(), 1);
        assert_eq!(reexported.mcp_servers["postgres"].command, "npx");
    }

    #[test]
    fn import_creates_skill_placeholders() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest {
            name: "demo".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            settings: Settings::default(),
            mcp_servers: Map::new(),
            skills: vec!["git-conventional-commits".to_string()],
            env_vars: vec![],
        };

        apply_manifest(&manifest, dir.path()).unwrap();

        assert!(dir
            .path()
            .join("skills/git-conventional-commits/SKILL.md")
            .exists());
    }

    #[test]
    fn export_only_includes_dotenv_names_never_values() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(".env"),
            "POSTGRES_PASSWORD=s3cr3t\nGITHUB_TOKEN=ghp_example\n",
        )
        .unwrap();

        let manifest = export_env(dir.path(), "demo", "1.0.0", None).unwrap();
        let yaml = serde_yaml::to_string(&manifest).unwrap();

        assert!(!yaml.contains("s3cr3t"));
        assert!(!yaml.contains("ghp_example"));
        assert_eq!(
            manifest.env_vars,
            vec!["GITHUB_TOKEN".to_string(), "POSTGRES_PASSWORD".to_string()]
        );
    }

    #[test]
    fn import_recreates_dotenv_with_blank_placeholders_and_preserves_existing_values() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".env"), "POSTGRES_PASSWORD=already-set\n").unwrap();

        let manifest = Manifest {
            name: "demo".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            settings: Settings::default(),
            mcp_servers: Map::new(),
            skills: vec![],
            env_vars: vec!["POSTGRES_PASSWORD".to_string(), "GITHUB_TOKEN".to_string()],
        };

        apply_manifest(&manifest, dir.path()).unwrap();

        let vars: BTreeMap<_, _> = env::load_env_file(dir.path())
            .unwrap()
            .into_iter()
            .collect();
        assert_eq!(vars["POSTGRES_PASSWORD"], "already-set");
        assert_eq!(vars["GITHUB_TOKEN"], "");
    }
}
