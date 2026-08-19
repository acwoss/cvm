use anyhow::Result;
use serde::Serialize;

use crate::env;
use crate::ui::account::{self, AccountInfo};
use crate::ui::config;
use crate::ui::marketplaces;
use crate::ui::skills;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentSummary {
    pub name: String,
    pub path: String,
    pub active: bool,
    pub description: Option<String>,
    pub plugin_count: usize,
    pub skills_agents_count: usize,
    pub account: Option<AccountInfo>,
}

/// Lista todo ambiente existente com seu caminho, se ele bate com o
/// `CVM_ENV` do processo atual, e um resumo agregado (descrição, contagem
/// de plugins/skills/agents, conta autenticada) para popular os cards da
/// listagem sem precisar abrir o ambiente. Erro em qualquer seção auxiliar
/// de um ambiente vira valor default (contador zero / `None`) - não
/// interrompe a listagem dos demais ambientes, mesmo tratamento adotado em
/// `environment_detail` (`ui/mod.rs`).
///
/// Ver a nota em `EnvironmentDetail::active` (Task 6) sobre essa não ser
/// uma verdade global entre sessões de shell.
pub fn list_environment_summaries() -> Result<Vec<EnvironmentSummary>> {
    let active = env::active_env();
    let mut summaries = Vec::new();
    for name in env::list_envs()? {
        let dir = env::env_dir(&name)?;

        let description = config::read_config_section(&dir)
            .ok()
            .and_then(|c| c.description);
        let plugin_count = marketplaces::list_marketplaces(&dir)
            .map(|ms| {
                ms.iter()
                    .flat_map(|m| &m.plugins)
                    .filter(|p| p.installed)
                    .count()
            })
            .unwrap_or(0);
        let skills_agents_count = skills::list_skills(&dir).map(|s| s.len()).unwrap_or(0)
            + skills::list_agents(&dir).map(|a| a.len()).unwrap_or(0);
        let account = account::read_account(&dir).ok().flatten();

        summaries.push(EnvironmentSummary {
            active: active.as_deref() == Some(name.as_str()),
            path: dir.display().to_string(),
            name,
            description,
            plugin_count,
            skills_agents_count,
            account,
        });
    }
    Ok(summaries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env as std_env;

    fn with_temp_home<F: FnOnce()>(f: F) {
        let _guard = crate::test_support::HOME_LOCK.lock().unwrap();
        let home = tempfile::tempdir().unwrap();
        let key = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
        // SAFETY: guardado por HOME_LOCK, sem outro teste lendo paths de home ao mesmo tempo.
        unsafe {
            std_env::set_var(key, home.path());
            std_env::set_var("CVM_USER_HOME", home.path());
            std_env::set_var("CVM_HOME", home.path().join(".cvm"));
        }
        f();
        unsafe {
            std_env::remove_var("CVM_HOME");
            std_env::remove_var("CVM_USER_HOME");
            std_env::remove_var("CVM_ENV");
        }
    }

    #[test]
    fn marks_the_environment_matching_cvm_env_as_active() {
        with_temp_home(|| {
            crate::env::create_env("work", true, false).unwrap();
            crate::env::create_env("personal", true, false).unwrap();
            // SAFETY: guardado por HOME_LOCK (mantido por with_temp_home).
            unsafe {
                std_env::set_var("CVM_ENV", "work");
            }

            let summaries = list_environment_summaries().unwrap();

            assert_eq!(summaries.len(), 2);
            let work = summaries.iter().find(|s| s.name == "work").unwrap();
            let personal = summaries.iter().find(|s| s.name == "personal").unwrap();
            assert!(work.active);
            assert!(!personal.active);
        });
    }

    #[test]
    fn returns_empty_list_when_no_environments_exist() {
        with_temp_home(|| {
            assert_eq!(list_environment_summaries().unwrap(), Vec::new());
        });
    }

    #[test]
    fn populates_description_plugin_count_skills_agents_count_and_account() {
        with_temp_home(|| {
            let (dir, _, _) = crate::env::create_env("work", true, false).unwrap();
            std::fs::write(
                dir.join("settings.json"),
                r#"{"description":"Ambiente de trabalho"}"#,
            )
            .unwrap();
            std::fs::write(
                dir.join(".claude.json"),
                r#"{"oauthAccount":{"emailAddress":"dev@example.com"}}"#,
            )
            .unwrap();
            std::fs::create_dir_all(dir.join("skills").join("my-skill")).unwrap();
            std::fs::write(
                dir.join("skills").join("my-skill").join("SKILL.md"),
                "---\nname: my-skill\ndescription: teste\n---\nbody",
            )
            .unwrap();

            let summaries = list_environment_summaries().unwrap();

            let work = summaries.iter().find(|s| s.name == "work").unwrap();
            assert_eq!(work.description.as_deref(), Some("Ambiente de trabalho"));
            assert_eq!(work.plugin_count, 0);
            assert_eq!(work.skills_agents_count, 1);
            assert_eq!(
                work.account.as_ref().unwrap().email.as_deref(),
                Some("dev@example.com")
            );
        });
    }

    #[test]
    fn defaults_to_zero_counts_none_account_and_none_description_for_a_bare_environment() {
        with_temp_home(|| {
            crate::env::create_env("bare", true, false).unwrap();

            let summaries = list_environment_summaries().unwrap();

            let bare = summaries.iter().find(|s| s.name == "bare").unwrap();
            assert_eq!(bare.description, None);
            assert_eq!(bare.plugin_count, 0);
            assert_eq!(bare.skills_agents_count, 0);
            assert_eq!(bare.account, None);
        });
    }
}
