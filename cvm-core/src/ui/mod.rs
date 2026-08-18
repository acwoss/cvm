mod account;
mod config;
mod hooks;
mod marketplaces;
mod skill_editor;
mod skills;
mod summary;

pub use account::{AccountInfo, AuthMethod, AuthStatus};
pub use config::{
    list_env_var_summaries, read_config_section, remove_env_var, reveal_value,
    write_config_section, write_env_var, ConfigSection, EnvVarSource, EnvVarSummary,
};
pub use hooks::{delete_hook, list_hooks, read_hook, set_hook_enabled, write_hook, HookSummary};
pub use marketplaces::{list_marketplaces, MarketplaceInfo, PluginInfo};
pub use skill_editor::{
    create_agent, create_skill, delete_agent, delete_skill, read_agent_content, read_skill_content,
    write_agent_content, write_skill_content, SkillContent,
};
pub use skills::{list_agents, list_skills, SkillOrAgentInfo};
pub use summary::{list_environment_summaries, EnvironmentSummary};

use anyhow::Result;
use serde::Serialize;

use crate::env;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentDetail {
    pub name: String,
    pub path: String,
    pub active: bool,
    pub config: ConfigSection,
    pub env_vars: Vec<EnvVarSummary>,
    pub marketplaces: Vec<MarketplaceInfo>,
    pub skills: Vec<SkillOrAgentInfo>,
    pub agents: Vec<SkillOrAgentInfo>,
    pub account: Option<AccountInfo>,
    /// Erros de parsing não fatais por seção (ex.: settings.json
    /// corrompido) - as outras seções continuam populadas normalmente.
    pub warnings: Vec<String>,
}

pub fn environment_detail(name: &str) -> Result<EnvironmentDetail> {
    let dir = env::ensure_env_exists(name)?;
    let mut warnings = Vec::new();

    let config = read_config_section(&dir).unwrap_or_else(|err| {
        warnings.push(format!("settings.json: {err:#}"));
        ConfigSection::default()
    });
    let env_vars = list_env_var_summaries(&dir).unwrap_or_else(|err| {
        warnings.push(format!("env vars: {err:#}"));
        Vec::new()
    });
    let marketplaces = list_marketplaces(&dir).unwrap_or_else(|err| {
        warnings.push(format!("marketplaces: {err:#}"));
        Vec::new()
    });
    let skills = list_skills(&dir).unwrap_or_else(|err| {
        warnings.push(format!("skills: {err:#}"));
        Vec::new()
    });
    let agents = list_agents(&dir).unwrap_or_else(|err| {
        warnings.push(format!("agents: {err:#}"));
        Vec::new()
    });
    let account = account::read_account(&dir).unwrap_or_else(|err| {
        warnings.push(format!("account: {err:#}"));
        None
    });

    Ok(EnvironmentDetail {
        active: env::active_env().as_deref() == Some(name),
        path: dir.display().to_string(),
        name: name.to_string(),
        config,
        env_vars,
        marketplaces,
        skills,
        agents,
        account,
        warnings,
    })
}

pub fn reveal_env_var(name: &str, source: EnvVarSource, key: &str) -> Result<String> {
    let dir = env::ensure_env_exists(name)?;
    config::reveal_value(&dir, source, key)
}

/// Roda `claude auth status` no ambiente `name` e retorna o resultado bruto.
/// Lê o stdout independente do código de saída - `claude` sai com 1 quando
/// não autenticado, mas ainda imprime um JSON válido (`loggedIn: false`),
/// que não é uma falha a propagar como erro.
pub fn check_auth_status(name: &str) -> Result<AuthStatus> {
    let output = env::run_claude_command(name, &["auth".to_string(), "status".to_string()])?;
    account::parse_auth_status(&output.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env as std_env;
    use std::fs;

    fn with_temp_home<F: FnOnce(&std::path::Path)>(f: F) {
        let _guard = crate::test_support::HOME_LOCK.lock().unwrap();
        let home = tempfile::tempdir().unwrap();
        let key = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
        // SAFETY: guardado por HOME_LOCK, sem outro teste lendo paths de home ao mesmo tempo.
        unsafe {
            std_env::set_var(key, home.path());
            std_env::set_var("CVM_USER_HOME", home.path());
            std_env::set_var("CVM_HOME", home.path().join(".cvm"));
        }
        f(home.path());
        unsafe {
            std_env::remove_var("CVM_HOME");
            std_env::remove_var("CVM_USER_HOME");
        }
    }

    #[test]
    fn composes_every_section_for_a_healthy_environment() {
        with_temp_home(|_home| {
            let (dir, _, _) = crate::env::create_env("work", true, false).unwrap();
            fs::write(
                dir.join(".claude.json"),
                r#"{"oauthAccount":{"emailAddress":"dev@example.com"}}"#,
            )
            .unwrap();

            let detail = environment_detail("work").unwrap();

            assert_eq!(detail.name, "work");
            assert!(detail.warnings.is_empty());
            assert_eq!(
                detail.account.unwrap().email.as_deref(),
                Some("dev@example.com")
            );
        });
    }

    #[test]
    fn keeps_other_sections_when_settings_json_is_corrupted() {
        with_temp_home(|_home| {
            let (dir, _, _) = crate::env::create_env("work", true, false).unwrap();
            fs::write(dir.join("settings.json"), "{not valid json").unwrap();

            let detail = environment_detail("work").unwrap();

            assert!(!detail.warnings.is_empty());
            assert!(
                detail.warnings[0].contains("settings.json")
                    || detail.warnings.iter().any(|w| w.contains("settings"))
            );
        });
    }

    #[test]
    fn reveal_env_var_delegates_to_the_named_environment() {
        with_temp_home(|_home| {
            let (dir, _, _) = crate::env::create_env("work", true, false).unwrap();
            fs::write(dir.join(".env"), "TOKEN=abc123\n").unwrap();

            let value = reveal_env_var("work", EnvVarSource::Dotenv, "TOKEN").unwrap();

            assert_eq!(value, "abc123");
        });
    }

    #[test]
    fn environment_detail_never_leaks_secret_values_outside_reveal() {
        with_temp_home(|_home| {
            let (dir, _, _) = crate::env::create_env("work", true, false).unwrap();
            fs::write(dir.join(".env"), "DOTENV_SECRET=leak-me-not\n").unwrap();
            fs::write(
                dir.join("settings.json"),
                r#"{"env":{"SETTINGS_SECRET":"also-leak-me-not"}}"#,
            )
            .unwrap();

            let detail = environment_detail("work").unwrap();
            let serialized = serde_json::to_string(&detail).unwrap();

            assert!(!serialized.contains("leak-me-not"));
            assert!(!serialized.contains("also-leak-me-not"));
            // The *names* are expected to appear (that's the whole point of the Env Vars tab).
            assert!(serialized.contains("DOTENV_SECRET"));
            assert!(serialized.contains("SETTINGS_SECRET"));
        });
    }
}
