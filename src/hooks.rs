use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

/// Directory holding global lifecycle hook scripts (`~/.cvm/hooks`), one
/// file per event, shared by every environment.
pub fn hooks_dir() -> Result<PathBuf> {
    Ok(crate::env::cvm_home()?.join("hooks"))
}

/// Path of `event`'s hook script inside `hooks_dir`: extensionless on Unix
/// (needs `chmod +x` and a shebang), `<event>.cmd` on Windows - the same
/// per-platform convention already used for environment shims in `bin/`.
fn hook_path(hooks_dir: &Path, event: &str) -> PathBuf {
    if cfg!(windows) {
        hooks_dir.join(format!("{event}.cmd"))
    } else {
        hooks_dir.join(event)
    }
}

/// Runs `event`'s hook if present in `hooks_dir`, propagating any failure.
/// For `pre-*` events: a failing hook must abort the operation in progress.
///
/// Only the unit tests below call this so far - the `pre-*` events are wired
/// into `remove_env`/`resolve_activate`/`resolve_deactivate` by the follow-up
/// lifecycle-hooks work. The expectation is scoped to non-test builds (the
/// tests do exercise it) and will itself start failing the build once a real
/// caller lands, so it can't outlive its purpose.
#[cfg_attr(not(test), expect(dead_code))]
pub fn run_pre_hook(hooks_dir: &Path, event: &str, env_name: &str, env_dir: &Path) -> Result<()> {
    execute_hook(hooks_dir, event, env_name, env_dir)
}

/// Runs `event`'s hook if present in `hooks_dir`, only warning on failure.
/// For `post-*` events: the operation already completed, so a failing hook
/// must not be treated as a `cvm` failure.
pub fn run_post_hook(hooks_dir: &Path, event: &str, env_name: &str, env_dir: &Path) {
    if let Err(err) = execute_hook(hooks_dir, event, env_name, env_dir) {
        eprintln!("warning: {err:#}");
    }
}

fn execute_hook(hooks_dir: &Path, event: &str, env_name: &str, env_dir: &Path) -> Result<()> {
    let path = hook_path(hooks_dir, event);
    if !path.is_file() {
        return Ok(());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path)
            .with_context(|| format!("failed to stat hook {}", path.display()))?
            .permissions()
            .mode();
        if mode & 0o111 == 0 {
            eprintln!(
                "warning: hook {} exists but is not executable (chmod +x it to enable), skipping",
                path.display()
            );
            return Ok(());
        }
    }

    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(&path);
        c
    } else {
        Command::new(&path)
    };

    cmd.env("CVM_HOOK_EVENT", event)
        .env("CVM_ENV", env_name)
        .env("CVM_ENV_PATH", env_dir);

    let status = cmd
        .status()
        .with_context(|| format!("failed to execute hook {}", path.display()))?;

    if !status.success() {
        bail!("hook '{event}' ({}) exited with {status}", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_unix_hook(path: &Path, script: &str) {
        fs::write(path, script).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
        }
    }

    #[test]
    fn missing_hook_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        execute_hook(dir.path(), "pre-activate", "work", Path::new("/envs/work")).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn hook_receives_expected_env_vars() {
        let dir = tempfile::tempdir().unwrap();
        let out_file = dir.path().join("out.txt");
        let hook = hook_path(dir.path(), "post-create");
        write_unix_hook(
            &hook,
            &format!(
                "#!/bin/sh\necho \"$CVM_HOOK_EVENT|$CVM_ENV|$CVM_ENV_PATH\" > {}\n",
                out_file.display()
            ),
        );

        execute_hook(dir.path(), "post-create", "work", Path::new("/envs/work")).unwrap();

        let contents = fs::read_to_string(&out_file).unwrap();
        assert_eq!(contents.trim(), "post-create|work|/envs/work");
    }

    #[cfg(unix)]
    #[test]
    fn execute_hook_propagates_nonzero_exit() {
        let dir = tempfile::tempdir().unwrap();
        let hook = hook_path(dir.path(), "pre-remove");
        write_unix_hook(&hook, "#!/bin/sh\nexit 1\n");

        let err =
            execute_hook(dir.path(), "pre-remove", "work", Path::new("/envs/work")).unwrap_err();
        assert!(err.to_string().contains("pre-remove"));
    }

    #[cfg(unix)]
    #[test]
    fn non_executable_hook_is_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let hook = hook_path(dir.path(), "post-create");
        fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o644)).unwrap();

        execute_hook(dir.path(), "post-create", "work", Path::new("/envs/work")).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn run_pre_hook_propagates_failure() {
        let dir = tempfile::tempdir().unwrap();
        let hook = hook_path(dir.path(), "pre-remove");
        write_unix_hook(&hook, "#!/bin/sh\nexit 1\n");

        assert!(run_pre_hook(dir.path(), "pre-remove", "work", Path::new("/envs/work")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn run_post_hook_swallows_failure() {
        let dir = tempfile::tempdir().unwrap();
        let hook = hook_path(dir.path(), "post-remove");
        write_unix_hook(&hook, "#!/bin/sh\nexit 1\n");

        // Must not panic - failure is only printed to stderr.
        run_post_hook(dir.path(), "post-remove", "work", Path::new("/envs/work"));
    }
}
