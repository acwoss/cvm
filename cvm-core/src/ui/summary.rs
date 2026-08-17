use anyhow::Result;
use serde::Serialize;

use crate::env;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EnvironmentSummary {
    pub name: String,
    pub path: String,
    pub active: bool,
}

/// Lista todo ambiente existente com seu caminho e se ele bate com o
/// `CVM_ENV` do processo atual. Ver a nota em `EnvironmentDetail::active`
/// (Task 6) sobre essa não ser uma verdade global entre sessões de shell.
pub fn list_environment_summaries() -> Result<Vec<EnvironmentSummary>> {
    let active = env::active_env();
    let mut summaries = Vec::new();
    for name in env::list_envs()? {
        let dir = env::env_dir(&name)?;
        summaries.push(EnvironmentSummary {
            active: active.as_deref() == Some(name.as_str()),
            path: dir.display().to_string(),
            name,
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
}
