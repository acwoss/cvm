use std::fs;

use anyhow::{bail, Context, Result};
use serde::Serialize;

/// Os 7 eventos de lifecycle que o `cvm` executa, na ordem em que
/// aparecem no ciclo de vida de um ambiente (ver `cvm-core/src/env.rs` e
/// a tabela do README). Hooks são sempre globais — nenhuma função deste
/// módulo recebe um diretório de ambiente.
pub const HOOK_EVENTS: [&str; 7] = [
    "post-create",
    "pre-activate",
    "post-activate",
    "pre-deactivate",
    "post-deactivate",
    "pre-remove",
    "post-remove",
];

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookSummary {
    pub event: String,
    pub configured: bool,
    /// No Unix, reflete o bit de execução real (o mesmo que `execute_hook`
    /// verifica antes de rodar um hook - ver `hooks.rs`). Fora do Unix não
    /// existe esse conceito (o `.cmd` sempre roda se presente), então
    /// `enabled` é só um espelho de `configured` nessas plataformas.
    pub enabled: bool,
    pub preview: Option<String>,
}

#[cfg(unix)]
fn is_executable(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(_path: &std::path::Path) -> bool {
    true
}

fn validate_event(event: &str) -> Result<()> {
    if !HOOK_EVENTS.contains(&event) {
        bail!("invalid hook event: '{event}'");
    }
    Ok(())
}

/// Primeira linha não-vazia e que não seja o shebang, truncada em 80
/// caracteres - usada como prévia do script na lista.
fn preview_line(content: &str) -> Option<String> {
    const MAX_LEN: usize = 80;
    let line = content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("#!"))?;
    if line.chars().count() > MAX_LEN {
        let truncated: String = line.chars().take(MAX_LEN).collect();
        Some(format!("{truncated}…"))
    } else {
        Some(line.to_string())
    }
}

pub fn list_hooks() -> Result<Vec<HookSummary>> {
    let dir = crate::hooks::hooks_dir()?;
    Ok(HOOK_EVENTS
        .iter()
        .map(|&event| {
            let path = crate::hooks::hook_path(&dir, event);
            // `is_file` (não o sucesso de `read_to_string`) para bater com o
            // critério real que `execute_hook` usa: um arquivo presente mas
            // não-UTF8 ou ilegível ainda é "configurado" de verdade (o cvm
            // vai tentar executá-lo no próximo lifecycle), só sem prévia.
            let configured = path.is_file();
            let preview = fs::read_to_string(&path)
                .ok()
                .as_deref()
                .and_then(preview_line);
            HookSummary {
                event: event.to_string(),
                configured,
                enabled: configured && is_executable(&path),
                preview,
            }
        })
        .collect())
}

pub fn read_hook(event: &str) -> Result<Option<String>> {
    validate_event(event)?;
    let dir = crate::hooks::hooks_dir()?;
    let path = crate::hooks::hook_path(&dir, event);
    match fs::read_to_string(&path) {
        Ok(content) => Ok(Some(content)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("failed to read {}", path.display())),
    }
}

pub fn write_hook(event: &str, content: &str) -> Result<()> {
    validate_event(event)?;
    let dir = crate::hooks::hooks_dir()?;
    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let path = crate::hooks::hook_path(&dir, event);
    fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)
            .with_context(|| format!("failed to stat {}", path.display()))?
            .permissions();
        // Só o bit de execução do dono - não amplia group/other além do que
        // já estava lá (evita, por exemplo, transformar um arquivo 0o600 em
        // 0o711 e dar execução a group/other que não tinham nem leitura).
        perms.set_mode(perms.mode() | 0o100);
        fs::set_permissions(&path, perms)
            .with_context(|| format!("failed to chmod +x {}", path.display()))?;
    }

    Ok(())
}

/// Liga/desliga um hook já configurado via o bit de execução real (o mesmo
/// que `execute_hook` verifica). Só existe no Unix - fora dele não há como
/// desabilitar um hook individualmente sem apagá-lo (o `.cmd` sempre roda se
/// presente), então retorna erro em vez de fingir uma ação sem efeito real.
pub fn set_hook_enabled(event: &str, enabled: bool) -> Result<()> {
    validate_event(event)?;
    let dir = crate::hooks::hooks_dir()?;
    let path = crate::hooks::hook_path(&dir, event);
    if !path.is_file() {
        bail!("hook '{event}' is not configured");
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)
            .with_context(|| format!("failed to stat {}", path.display()))?
            .permissions();
        let mode = perms.mode();
        perms.set_mode(if enabled { mode | 0o100 } else { mode & !0o111 });
        fs::set_permissions(&path, perms)
            .with_context(|| format!("failed to chmod {}", path.display()))?;
        Ok(())
    }

    #[cfg(not(unix))]
    {
        let _ = enabled;
        bail!("desabilitar hooks individualmente não é suportado nesta plataforma")
    }
}

pub fn delete_hook(event: &str) -> Result<()> {
    validate_event(event)?;
    let dir = crate::hooks::hooks_dir()?;
    let path = crate::hooks::hook_path(&dir, event);
    fs::remove_file(&path).with_context(|| format!("failed to remove {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env as std_env;

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
    fn list_hooks_always_returns_the_seven_events_in_order() {
        with_temp_home(|_home| {
            let hooks = list_hooks().unwrap();
            let events: Vec<&str> = hooks.iter().map(|h| h.event.as_str()).collect();
            assert_eq!(events, HOOK_EVENTS.to_vec());
            assert!(hooks
                .iter()
                .all(|h| !h.configured && !h.enabled && h.preview.is_none()));
        });
    }

    #[test]
    fn list_hooks_reports_configured_and_preview_when_a_script_exists() {
        with_temp_home(|_home| {
            write_hook("post-create", "#!/bin/sh\n\necho hello\n").unwrap();

            let hooks = list_hooks().unwrap();
            let entry = hooks.iter().find(|h| h.event == "post-create").unwrap();

            assert!(entry.configured);
            assert_eq!(entry.preview.as_deref(), Some("echo hello"));
        });
    }

    #[test]
    fn read_hook_returns_none_when_not_configured() {
        with_temp_home(|_home| {
            assert_eq!(read_hook("pre-activate").unwrap(), None);
        });
    }

    #[test]
    fn read_hook_rejects_invalid_event() {
        with_temp_home(|_home| {
            assert!(read_hook("not-a-real-event").is_err());
        });
    }

    #[test]
    fn write_hook_then_read_hook_round_trips_content() {
        with_temp_home(|_home| {
            write_hook("post-remove", "#!/bin/sh\necho bye\n").unwrap();

            assert_eq!(
                read_hook("post-remove").unwrap().as_deref(),
                Some("#!/bin/sh\necho bye\n")
            );
        });
    }

    #[cfg(unix)]
    #[test]
    fn write_hook_makes_the_script_executable() {
        use std::os::unix::fs::PermissionsExt;

        with_temp_home(|_home| {
            write_hook("pre-remove", "#!/bin/sh\nexit 0\n").unwrap();

            let dir = crate::hooks::hooks_dir().unwrap();
            let path = crate::hooks::hook_path(&dir, "pre-remove");
            let mode = fs::metadata(&path).unwrap().permissions().mode();
            assert_ne!(mode & 0o111, 0, "hook deve ficar executável após salvar");
        });
    }

    #[cfg(unix)]
    #[test]
    fn list_hooks_reports_configured_even_when_content_is_not_valid_utf8() {
        with_temp_home(|_home| {
            let dir = crate::hooks::hooks_dir().unwrap();
            fs::create_dir_all(&dir).unwrap();
            let path = crate::hooks::hook_path(&dir, "post-create");
            fs::write(&path, [0xFF, 0xFE, 0x00]).unwrap();

            let hooks = list_hooks().unwrap();
            let entry = hooks.iter().find(|h| h.event == "post-create").unwrap();

            assert!(
                entry.configured,
                "um arquivo presente mas não-UTF8 ainda é executado pelo cvm, então deve contar como configurado"
            );
            assert_eq!(entry.preview, None);
        });
    }

    #[cfg(unix)]
    #[test]
    fn write_hook_does_not_grant_group_or_other_execute() {
        use std::os::unix::fs::PermissionsExt;

        with_temp_home(|_home| {
            write_hook("pre-remove", "#!/bin/sh\nexit 0\n").unwrap();

            let dir = crate::hooks::hooks_dir().unwrap();
            let path = crate::hooks::hook_path(&dir, "pre-remove");
            let mode = fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(
                mode & 0o011,
                0,
                "não deve conceder execução a group/other, só ao dono"
            );
        });
    }

    #[cfg(unix)]
    #[test]
    fn set_hook_enabled_false_removes_execute_bit_and_true_restores_it() {
        use std::os::unix::fs::PermissionsExt;

        with_temp_home(|_home| {
            write_hook("post-activate", "#!/bin/sh\nexit 0\n").unwrap();
            let dir = crate::hooks::hooks_dir().unwrap();
            let path = crate::hooks::hook_path(&dir, "post-activate");

            set_hook_enabled("post-activate", false).unwrap();
            let mode = fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o111, 0);
            assert!(
                !list_hooks()
                    .unwrap()
                    .iter()
                    .find(|h| h.event == "post-activate")
                    .unwrap()
                    .enabled
            );

            set_hook_enabled("post-activate", true).unwrap();
            let mode = fs::metadata(&path).unwrap().permissions().mode();
            assert_ne!(mode & 0o111, 0);
            assert!(
                list_hooks()
                    .unwrap()
                    .iter()
                    .find(|h| h.event == "post-activate")
                    .unwrap()
                    .enabled
            );
        });
    }

    #[test]
    fn set_hook_enabled_fails_when_not_configured() {
        with_temp_home(|_home| {
            assert!(set_hook_enabled("post-activate", false).is_err());
        });
    }

    #[test]
    fn delete_hook_removes_the_file() {
        with_temp_home(|_home| {
            write_hook("post-activate", "#!/bin/sh\n").unwrap();

            delete_hook("post-activate").unwrap();

            assert_eq!(read_hook("post-activate").unwrap(), None);
        });
    }

    #[test]
    fn delete_hook_fails_when_not_configured() {
        with_temp_home(|_home| {
            assert!(delete_hook("pre-deactivate").is_err());
        });
    }

    #[test]
    fn write_hook_rejects_invalid_event() {
        with_temp_home(|_home| {
            assert!(write_hook("not-a-real-event", "x").is_err());
        });
    }
}
