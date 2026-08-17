use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct PluginInfo {
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub installed: bool,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceInfo {
    pub id: String,
    pub repo: Option<String>,
    pub plugins: Vec<PluginInfo>,
}

#[derive(Debug, Deserialize)]
struct KnownMarketplaceEntry {
    source: MarketplaceSource,
}

#[derive(Debug, Deserialize)]
struct MarketplaceSource {
    repo: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct MarketplaceManifest {
    #[serde(default)]
    plugins: Vec<MarketplacePluginEntry>,
}

#[derive(Debug, Deserialize)]
struct MarketplacePluginEntry {
    name: String,
    #[serde(default)]
    description: Option<String>,
}

pub fn list_marketplaces(env_dir: &Path) -> Result<Vec<MarketplaceInfo>> {
    let known_path = env_dir.join("plugins/known_marketplaces.json");
    if !known_path.is_file() {
        return Ok(Vec::new());
    }
    let known: BTreeMap<String, KnownMarketplaceEntry> = serde_json::from_str(
        &fs::read_to_string(&known_path)
            .with_context(|| format!("failed to read {}", known_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", known_path.display()))?;

    let enabled = read_enabled_plugins(env_dir)?;
    let installed = read_installed_plugin_versions(env_dir)?;

    let mut marketplaces = Vec::new();
    for (id, entry) in known {
        let manifest_path = env_dir
            .join("plugins/marketplaces")
            .join(&id)
            .join(".claude-plugin/marketplace.json");
        let manifest: MarketplaceManifest = if manifest_path.is_file() {
            serde_json::from_str(
                &fs::read_to_string(&manifest_path)
                    .with_context(|| format!("failed to read {}", manifest_path.display()))?,
            )
            .with_context(|| format!("failed to parse {}", manifest_path.display()))?
        } else {
            MarketplaceManifest::default()
        };

        let plugins = manifest
            .plugins
            .into_iter()
            .map(|p| {
                let key = format!("{}@{id}", p.name);
                PluginInfo {
                    enabled: enabled.get(&key).copied().unwrap_or(false),
                    installed: installed.contains_key(&key),
                    version: installed.get(&key).cloned(),
                    name: p.name,
                    description: p.description,
                }
            })
            .collect();

        marketplaces.push(MarketplaceInfo {
            repo: entry.source.repo,
            plugins,
            id,
        });
    }
    marketplaces.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(marketplaces)
}

fn read_enabled_plugins(env_dir: &Path) -> Result<BTreeMap<String, bool>> {
    let path = env_dir.join("settings.json");
    if !path.is_file() {
        return Ok(BTreeMap::new());
    }
    let value: Value = serde_json::from_str(
        &fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(value
        .get("enabledPlugins")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_bool().map(|b| (k.clone(), b)))
                .collect()
        })
        .unwrap_or_default())
}

fn read_installed_plugin_versions(env_dir: &Path) -> Result<BTreeMap<String, String>> {
    let path = env_dir.join("plugins/installed_plugins.json");
    if !path.is_file() {
        return Ok(BTreeMap::new());
    }
    let value: Value = serde_json::from_str(
        &fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(value
        .get("plugins")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, versions)| {
                    let version = versions.as_array()?.first()?.get("version")?.as_str()?;
                    Some((k.clone(), version.to_string()))
                })
                .collect()
        })
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_fixture(dir: &Path) {
        fs::write(
            dir.join("plugins/known_marketplaces.json"),
            r#"{"acme":{"source":{"source":"github","repo":"acme/marketplace"},"installLocation":"/x","lastUpdated":"2026-01-01T00:00:00Z"}}"#,
        )
        .unwrap();
        fs::create_dir_all(dir.join("plugins/marketplaces/acme/.claude-plugin")).unwrap();
        fs::write(
            dir.join("plugins/marketplaces/acme/.claude-plugin/marketplace.json"),
            r#"{"name":"acme","plugins":[{"name":"tool","description":"A tool"},{"name":"other"}]}"#,
        )
        .unwrap();
        fs::write(
            dir.join("plugins/installed_plugins.json"),
            r#"{"version":2,"plugins":{"tool@acme":[{"scope":"user","installPath":"/x","version":"1.2.3"}]}}"#,
        )
        .unwrap();
        fs::write(
            dir.join("settings.json"),
            r#"{"enabledPlugins":{"tool@acme":true,"other@acme":false}}"#,
        )
        .unwrap();
    }

    #[test]
    fn lists_marketplace_with_cross_referenced_plugin_state() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("plugins")).unwrap();
        write_fixture(dir.path());

        let marketplaces = list_marketplaces(dir.path()).unwrap();

        assert_eq!(marketplaces.len(), 1);
        let acme = &marketplaces[0];
        assert_eq!(acme.id, "acme");
        assert_eq!(acme.repo.as_deref(), Some("acme/marketplace"));
        assert_eq!(acme.plugins.len(), 2);

        let tool = acme.plugins.iter().find(|p| p.name == "tool").unwrap();
        assert!(tool.enabled);
        assert!(tool.installed);
        assert_eq!(tool.version.as_deref(), Some("1.2.3"));
        assert_eq!(tool.description.as_deref(), Some("A tool"));

        let other = acme.plugins.iter().find(|p| p.name == "other").unwrap();
        assert!(!other.enabled);
        assert!(!other.installed);
        assert_eq!(other.version, None);
    }

    #[test]
    fn returns_empty_list_when_no_marketplaces_known() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(list_marketplaces(dir.path()).unwrap().len(), 0);
    }
}
