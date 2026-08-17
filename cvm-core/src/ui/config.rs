use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::env;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
    obj.remove("env");

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

fn modify_settings_json(
    env_dir: &Path,
    f: impl FnOnce(&mut serde_json::Map<String, Value>) -> Result<()>,
) -> Result<()> {
    let path = env_dir.join("settings.json");
    let mut value: Value = if path.is_file() {
        serde_json::from_str(
            &fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?,
        )
        .with_context(|| format!("failed to parse {}", path.display()))?
    } else {
        Value::Object(serde_json::Map::new())
    };
    let obj = value
        .as_object_mut()
        .context("settings.json must be a JSON object")?;
    f(obj)?;
    fs::write(&path, serde_json::to_string_pretty(&value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

pub fn write_config_section(
    env_dir: &Path,
    allowed_tools: &[String],
    denied_tools: &[String],
) -> Result<()> {
    modify_settings_json(env_dir, |obj| {
        let permissions_obj = obj
            .entry("permissions")
            .or_insert_with(|| Value::Object(serde_json::Map::new()))
            .as_object_mut()
            .context("settings.json permissions must be an object")?;
        permissions_obj.insert(
            "allow".to_string(),
            serde_json::to_value(allowed_tools).context("failed to serialize allowed tools")?,
        );
        permissions_obj.insert(
            "deny".to_string(),
            serde_json::to_value(denied_tools).context("failed to serialize denied tools")?,
        );
        Ok(())
    })
}

pub fn write_env_var(env_dir: &Path, source: EnvVarSource, key: &str, value: &str) -> Result<()> {
    match source {
        EnvVarSource::Dotenv => {
            let mut vars: std::collections::BTreeMap<String, String> =
                env::load_env_file(env_dir)?.into_iter().collect();
            vars.insert(key.to_string(), value.to_string());
            env::write_env_file(env_dir, &vars)
        }
        EnvVarSource::Settings => modify_settings_json(env_dir, |obj| {
            let env_obj = obj
                .entry("env")
                .or_insert_with(|| Value::Object(serde_json::Map::new()))
                .as_object_mut()
                .context("settings.json env must be an object")?;
            env_obj.insert(key.to_string(), Value::String(value.to_string()));
            Ok(())
        }),
    }
}

pub fn remove_env_var(env_dir: &Path, source: EnvVarSource, key: &str) -> Result<()> {
    match source {
        EnvVarSource::Dotenv => {
            let mut vars: std::collections::BTreeMap<String, String> =
                env::load_env_file(env_dir)?.into_iter().collect();
            vars.remove(key);
            env::write_env_file(env_dir, &vars)
        }
        EnvVarSource::Settings => modify_settings_json(env_dir, |obj| {
            if let Some(env_obj) = obj.get_mut("env").and_then(|v| v.as_object_mut()) {
                env_obj.remove(key);
            }
            Ok(())
        }),
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

    #[test]
    fn read_config_section_strips_the_env_block_from_other() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("settings.json"),
            r#"{"env":{"SECRET_KEY":"super-secret"},"theme":"dark"}"#,
        )
        .unwrap();

        let config = read_config_section(dir.path()).unwrap();

        assert!(config.other.get("env").is_none());
        assert_eq!(config.other["theme"], "dark");
    }

    #[test]
    fn write_config_section_updates_permissions_and_preserves_other_keys() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("settings.json"),
            r#"{"theme":"dark","permissions":{"allow":["old-tool"]}}"#,
        )
        .unwrap();

        write_config_section(
            dir.path(),
            &["read-file".to_string(), "write-file".to_string()],
            &["exec".to_string()],
        )
        .unwrap();

        let raw = fs::read_to_string(dir.path().join("settings.json")).unwrap();
        let value: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(value["theme"], "dark");
        assert_eq!(
            value["permissions"]["allow"],
            serde_json::json!(["read-file", "write-file"])
        );
        assert_eq!(value["permissions"]["deny"], serde_json::json!(["exec"]));
    }

    #[test]
    fn write_config_section_creates_settings_json_when_missing() {
        let dir = tempfile::tempdir().unwrap();

        write_config_section(dir.path(), &["read-file".to_string()], &[]).unwrap();

        let config = read_config_section(dir.path()).unwrap();
        assert_eq!(config.allowed_tools, vec!["read-file".to_string()]);
    }

    #[test]
    fn write_env_var_adds_to_dotenv() {
        let dir = tempfile::tempdir().unwrap();

        write_env_var(dir.path(), EnvVarSource::Dotenv, "NEW_VAR", "hello").unwrap();

        let value = reveal_value(dir.path(), EnvVarSource::Dotenv, "NEW_VAR").unwrap();
        assert_eq!(value, "hello");
    }

    #[test]
    fn write_env_var_adds_to_settings_json_env_block() {
        let dir = tempfile::tempdir().unwrap();

        write_env_var(dir.path(), EnvVarSource::Settings, "NEW_VAR", "hello").unwrap();

        let value = reveal_value(dir.path(), EnvVarSource::Settings, "NEW_VAR").unwrap();
        assert_eq!(value, "hello");
    }

    #[test]
    fn write_env_var_preserves_other_dotenv_entries() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".env"), "EXISTING=value\n").unwrap();

        write_env_var(dir.path(), EnvVarSource::Dotenv, "NEW_VAR", "hello").unwrap();

        let existing = reveal_value(dir.path(), EnvVarSource::Dotenv, "EXISTING").unwrap();
        assert_eq!(existing, "value");
    }

    #[test]
    fn remove_env_var_removes_from_dotenv() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".env"), "TO_REMOVE=value\nKEEP=other\n").unwrap();

        remove_env_var(dir.path(), EnvVarSource::Dotenv, "TO_REMOVE").unwrap();

        let vars = list_env_var_summaries(dir.path()).unwrap();
        assert!(!vars.iter().any(|v| v.key == "TO_REMOVE"));
        assert!(vars.iter().any(|v| v.key == "KEEP"));
    }

    #[test]
    fn remove_env_var_removes_from_settings_json_env_block() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("settings.json"),
            r#"{"env":{"TO_REMOVE":"value","KEEP":"other"}}"#,
        )
        .unwrap();

        remove_env_var(dir.path(), EnvVarSource::Settings, "TO_REMOVE").unwrap();

        let vars = list_env_var_summaries(dir.path()).unwrap();
        assert!(!vars
            .iter()
            .any(|v| v.key == "TO_REMOVE" && v.source == EnvVarSource::Settings));
        assert!(vars
            .iter()
            .any(|v| v.key == "KEEP" && v.source == EnvVarSource::Settings));
    }
}
