use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::env;

#[derive(Debug, Clone, Default, Serialize)]
pub struct ConfigSection {
    pub allowed_tools: Vec<String>,
    pub denied_tools: Vec<String>,
    /// Todas as outras chaves de settings.json (enabledPlugins,
    /// extraKnownMarketplaces, pluginConfigs, theme, effortLevel, ...),
    /// expostas como estão para um visualizador JSON genérico. Ver
    /// "Global Constraints" sobre `pluginConfigs` não ser mascarado.
    pub other: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnvVarSource {
    Dotenv,
    Settings,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EnvVarSummary {
    pub key: String,
    pub source: EnvVarSource,
}

pub fn read_config_section(env_dir: &Path) -> Result<ConfigSection> {
    let path = env_dir.join("settings.json");
    if !path.is_file() {
        return Ok(ConfigSection::default());
    }
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut value: Value = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    let obj = value
        .as_object_mut()
        .context("settings.json must be a JSON object")?;

    let (allowed_tools, denied_tools) = match obj.remove("permissions") {
        Some(Value::Object(mut perms)) => (
            take_string_array(&mut perms, "allow"),
            take_string_array(&mut perms, "deny"),
        ),
        _ => (Vec::new(), Vec::new()),
    };
    obj.remove("mcpServers");

    Ok(ConfigSection {
        allowed_tools,
        denied_tools,
        other: value,
    })
}

fn take_string_array(obj: &mut serde_json::Map<String, Value>, key: &str) -> Vec<String> {
    obj.remove(key)
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect()
}

pub fn list_env_var_summaries(env_dir: &Path) -> Result<Vec<EnvVarSummary>> {
    let mut vars: Vec<EnvVarSummary> = env::load_env_file(env_dir)?
        .into_iter()
        .map(|(key, _)| EnvVarSummary {
            key,
            source: EnvVarSource::Dotenv,
        })
        .collect();

    let path = env_dir.join("settings.json");
    if path.is_file() {
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let value: Value = serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(Value::Object(env_obj)) = value.get("env") {
            for key in env_obj.keys() {
                vars.push(EnvVarSummary {
                    key: key.clone(),
                    source: EnvVarSource::Settings,
                });
            }
        }
    }
    Ok(vars)
}

pub fn reveal_value(env_dir: &Path, source: EnvVarSource, key: &str) -> Result<String> {
    match source {
        EnvVarSource::Dotenv => env::load_env_file(env_dir)?
            .into_iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
            .with_context(|| format!("'{key}' not found in .env")),
        EnvVarSource::Settings => {
            let path = env_dir.join("settings.json");
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let value: Value = serde_json::from_str(&raw)
                .with_context(|| format!("failed to parse {}", path.display()))?;
            value
                .get("env")
                .and_then(|env| env.get(key))
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .with_context(|| format!("'{key}' not found in settings.json env block"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_config_section_extracts_permissions_and_keeps_other_keys_raw() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("settings.json"),
            r#"{"permissions":{"allow":["read-file"],"deny":["exec"]},"mcpServers":{},"theme":"dark","enabledPlugins":{"git@acme":true}}"#,
        )
        .unwrap();

        let config = read_config_section(dir.path()).unwrap();

        assert_eq!(config.allowed_tools, vec!["read-file".to_string()]);
        assert_eq!(config.denied_tools, vec!["exec".to_string()]);
        assert_eq!(config.other["theme"], "dark");
        assert_eq!(config.other["enabledPlugins"]["git@acme"], true);
        assert!(config.other.get("permissions").is_none());
        assert!(config.other.get("mcpServers").is_none());
    }

    #[test]
    fn read_config_section_defaults_when_settings_json_missing() {
        let dir = tempfile::tempdir().unwrap();
        let config = read_config_section(dir.path()).unwrap();
        assert!(config.allowed_tools.is_empty());
        assert!(config.denied_tools.is_empty());
    }

    #[test]
    fn list_env_var_summaries_merges_dotenv_and_settings_json_env_block() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".env"), "GITHUB_TOKEN=ghp_example\n").unwrap();
        fs::write(
            dir.path().join("settings.json"),
            r#"{"env":{"LANGFUSE_SECRET_KEY":"sk-lf-example"}}"#,
        )
        .unwrap();

        let mut vars = list_env_var_summaries(dir.path()).unwrap();
        vars.sort_by(|a, b| a.key.cmp(&b.key));

        assert_eq!(
            vars,
            vec![
                EnvVarSummary {
                    key: "GITHUB_TOKEN".to_string(),
                    source: EnvVarSource::Dotenv
                },
                EnvVarSummary {
                    key: "LANGFUSE_SECRET_KEY".to_string(),
                    source: EnvVarSource::Settings
                },
            ]
        );
    }

    #[test]
    fn reveal_value_reads_dotenv_value() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".env"), "GITHUB_TOKEN=ghp_example\n").unwrap();

        let value = reveal_value(dir.path(), EnvVarSource::Dotenv, "GITHUB_TOKEN").unwrap();

        assert_eq!(value, "ghp_example");
    }

    #[test]
    fn reveal_value_reads_settings_json_env_value() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("settings.json"),
            r#"{"env":{"LANGFUSE_SECRET_KEY":"sk-lf-example"}}"#,
        )
        .unwrap();

        let value =
            reveal_value(dir.path(), EnvVarSource::Settings, "LANGFUSE_SECRET_KEY").unwrap();

        assert_eq!(value, "sk-lf-example");
    }

    #[test]
    fn reveal_value_errors_when_key_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(reveal_value(dir.path(), EnvVarSource::Dotenv, "MISSING").is_err());
    }
}
