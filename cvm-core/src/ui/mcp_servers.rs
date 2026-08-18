use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::manifest::McpServer;
use crate::ui::marketplaces::list_enabled_plugin_dirs;
use crate::ui::plugin_source::ItemSource;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerInfo {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    /// Apenas os *nomes* das variáveis de ambiente do server - nunca os
    /// valores, que podem conter segredos (tokens, chaves de API). Mesmo
    /// tratamento de `EnvVarSummary` em `config.rs`.
    pub env_keys: Vec<String>,
    pub source: ItemSource,
}

fn env_keys(env: Option<BTreeMap<String, String>>) -> Vec<String> {
    env.map(|vars| vars.into_keys().collect()).unwrap_or_default()
}

fn read_native_mcp_servers(env_dir: &Path) -> Result<BTreeMap<String, McpServer>> {
    let path = env_dir.join("settings.json");
    if !path.is_file() {
        return Ok(BTreeMap::new());
    }
    let raw = fs::read_to_string(&path)?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    let Some(servers) = value.get("mcpServers") else {
        return Ok(BTreeMap::new());
    };
    Ok(serde_json::from_value(servers.clone()).unwrap_or_default())
}

pub fn list_mcp_servers(env_dir: &Path) -> Result<Vec<McpServerInfo>> {
    let mut servers = Vec::new();

    for (name, server) in read_native_mcp_servers(env_dir)? {
        servers.push(McpServerInfo {
            name,
            command: server.command,
            args: server.args,
            env_keys: env_keys(server.env),
            source: ItemSource::Native,
        });
    }

    for plugin_dir in list_enabled_plugin_dirs(env_dir)? {
        let mcp_path = plugin_dir.path.join(".mcp.json");
        let Ok(raw) = fs::read_to_string(&mcp_path) else {
            continue;
        };
        let Ok(map) = serde_json::from_str::<BTreeMap<String, McpServer>>(&raw) else {
            continue;
        };
        for (name, server) in map {
            servers.push(McpServerInfo {
                name,
                command: server.command,
                args: server.args,
                env_keys: env_keys(server.env),
                source: ItemSource::Plugin {
                    marketplace: plugin_dir.marketplace.clone(),
                    plugin: plugin_dir.plugin.clone(),
                },
            });
        }
    }

    servers.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(servers)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_plugin_fixture(dir: &Path, mcp_json: &str) {
        fs::create_dir_all(dir.join("plugins")).unwrap();
        fs::write(
            dir.join("plugins/known_marketplaces.json"),
            r#"{"acme":{"source":{"source":"github","repo":"acme/marketplace"},"installLocation":"/x","lastUpdated":"2026-01-01T00:00:00Z"}}"#,
        )
        .unwrap();
        fs::create_dir_all(dir.join("plugins/marketplaces/acme/.claude-plugin")).unwrap();
        fs::write(
            dir.join("plugins/marketplaces/acme/.claude-plugin/marketplace.json"),
            r#"{"name":"acme","plugins":[{"name":"tool"}]}"#,
        )
        .unwrap();
        fs::write(
            dir.join("plugins/installed_plugins.json"),
            r#"{"version":2,"plugins":{"tool@acme":[{"scope":"user","installPath":"/x","version":"1.2.3"}]}}"#,
        )
        .unwrap();
        fs::write(
            dir.join("settings.json"),
            r#"{"enabledPlugins":{"tool@acme":true}}"#,
        )
        .unwrap();
        let plugin_dir = dir.join("plugins/cache/acme/tool/1.2.3");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(plugin_dir.join(".mcp.json"), mcp_json).unwrap();
    }

    #[test]
    fn lists_native_mcp_server_from_settings_json() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("settings.json"),
            r#"{"mcpServers":{"postgres":{"command":"npx","args":["-y","@modelcontextprotocol/server-postgres"]}}}"#,
        )
        .unwrap();

        let servers = list_mcp_servers(dir.path()).unwrap();

        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "postgres");
        assert_eq!(servers[0].command, "npx");
        assert_eq!(servers[0].source, ItemSource::Native);
    }

    #[test]
    fn lists_plugin_provided_mcp_server_from_mcp_json() {
        let dir = tempfile::tempdir().unwrap();
        write_plugin_fixture(
            dir.path(),
            r#"{"memory-mcp":{"command":"uvx","args":["--from","${CLAUDE_PLUGIN_ROOT}","memory-mcp"]}}"#,
        );

        let servers = list_mcp_servers(dir.path()).unwrap();

        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "memory-mcp");
        assert_eq!(servers[0].command, "uvx");
        assert_eq!(
            servers[0].source,
            ItemSource::Plugin {
                marketplace: "acme".to_string(),
                plugin: "tool".to_string()
            }
        );
    }

    #[test]
    fn skips_malformed_mcp_json_without_failing() {
        let dir = tempfile::tempdir().unwrap();
        write_plugin_fixture(dir.path(), "{not valid json");

        let servers = list_mcp_servers(dir.path()).unwrap();

        assert_eq!(servers.len(), 0);
    }

    #[test]
    fn returns_empty_list_when_settings_json_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(list_mcp_servers(dir.path()).unwrap(), Vec::new());
    }

    #[test]
    fn exposes_env_var_names_but_never_their_values() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("settings.json"),
            r#"{"mcpServers":{"api":{"command":"npx","env":{"API_KEY":"sk-live-secret"}}}}"#,
        )
        .unwrap();

        let servers = list_mcp_servers(dir.path()).unwrap();
        let serialized = serde_json::to_string(&servers).unwrap();

        assert_eq!(servers[0].env_keys, vec!["API_KEY".to_string()]);
        assert!(!serialized.contains("sk-live-secret"));
        assert!(serialized.contains("API_KEY"));
    }
}
