mod account;
mod config;
mod marketplaces;
mod skills;
mod summary;

pub use account::AccountInfo;
pub use config::{
    list_env_var_summaries, read_config_section, reveal_value, ConfigSection, EnvVarSource,
    EnvVarSummary,
};
pub use marketplaces::{list_marketplaces, MarketplaceInfo, PluginInfo};
pub use skills::{list_agents, list_skills, SkillOrAgentInfo};
pub use summary::{list_environment_summaries, EnvironmentSummary};

use anyhow::Result;
use serde::Serialize;

use crate::env;

#[derive(Debug, Clone, Serialize)]
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
}
