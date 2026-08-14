use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use anyhow::{bail, Context, Result};

use crate::hooks;
use crate::shims::{env_bin_dir, write_env_shims};

/// Name of an environment's local dotenv file, relative to its directory.
const DOTENV_FILE: &str = ".env";

/// Name of Claude Code's OAuth credentials file. Copied from the global
/// config directory into a new environment on `cvm create` unless
/// `--anonymous` is passed; never touched by `export`/`import` (see
/// `manifest.rs`).
const CREDENTIALS_FILE: &str = ".credentials.json";

/// Env var pointing Claude Code at an isolated config directory.
pub const CONFIG_DIR_VAR: &str = "CLAUDE_CONFIG_DIR";
/// Env var identifying which cvm environment a process is running under.
/// Set shell-wide by `cvm use`/`activate` (via the shell hook), or scoped to
/// a single child process by `cvm run`/`cvm open` - either way, `cvm current`
/// and statusline integrations can read it the same way.
pub const ACTIVE_ENV_VAR: &str = "CVM_ENV";

#[derive(Debug, Default, PartialEq, Eq)]
pub struct InheritStats {
    pub skills_linked: usize,
    pub skills_copied: usize,
    pub settings_copied: bool,
}

/// Resolves the user home directory for cvm paths.
///
/// Order: `CVM_USER_HOME`, then `HOME`, then `USERPROFILE`, then `dirs::home_dir()`.
/// Preferring explicit env vars matters on Windows, where `dirs::home_dir()` uses
/// the Known Folder API and ignores `USERPROFILE` — which would break tests and
/// intentional overrides.
fn user_home() -> Result<PathBuf> {
    for key in ["CVM_USER_HOME", "HOME", "USERPROFILE"] {
        if let Some(value) = env::var_os(key).filter(|value| !value.is_empty()) {
            return Ok(PathBuf::from(value));
        }
    }
    dirs::home_dir().context("could not determine the home directory")
}

pub fn cvm_home() -> Result<PathBuf> {
    if let Some(home) = env::var_os("CVM_HOME").filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(home));
    }
    Ok(user_home()?.join(".cvm"))
}

pub fn envs_dir() -> Result<PathBuf> {
    Ok(cvm_home()?.join("envs"))
}

/// The global Claude Code config directory (`~/.claude`), regardless of any
/// `CLAUDE_CONFIG_DIR` override currently in effect - this is where a new
/// environment's credentials are copied from, not wherever the active
/// environment happens to point.
fn global_claude_dir() -> Result<PathBuf> {
    Ok(user_home()?.join(".claude"))
}

pub fn env_dir(name: &str) -> Result<PathBuf> {
    validate_name(name)?;
    Ok(envs_dir()?.join(name))
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\']) {
        bail!("invalid environment name: '{name}'");
    }
    Ok(())
}

/// Name of the environment active in the current process's environment, if any.
pub fn active_env() -> Option<String> {
    env::var(ACTIVE_ENV_VAR).ok().filter(|s| !s.is_empty())
}

/// Creates a new, empty environment directory. Unless `anonymous` is set,
/// also copies the global Claude Code credentials file into it (if one
/// exists) so the new environment starts out already logged in. Returns the
/// environment directory, whether credentials were copied, and inheritance
/// statistics.
pub fn create_env(
    name: &str,
    anonymous: bool,
    inherit: bool,
) -> Result<(PathBuf, bool, InheritStats)> {
    let dir = env_dir(name)?;
    if dir.exists() {
        bail!("environment '{name}' already exists at {}", dir.display());
    }
    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    let credentials_copied = if anonymous {
        false
    } else {
        copy_credentials(&global_claude_dir()?, &dir)?
    };

    ensure_env_layout(&dir)?;
    let inherit_stats = if inherit {
        inherit_from_global(&dir)?
    } else {
        InheritStats::default()
    };

    hooks::run_post_hook(&hooks::hooks_dir()?, "post-create", name, &dir);

    Ok((dir, credentials_copied, inherit_stats))
}

/// Ensures an environment directory has the standard layout: `skills/`, `bin/`,
/// and a starter `.env` file.
pub fn ensure_env_layout(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir.join("skills"))?;
    ensure_env_bin(dir)?;
    let _ = ensure_dotenv_file(dir)?;
    Ok(())
}

pub fn inherit_from_global(env_dir: &Path) -> Result<InheritStats> {
    let global = global_claude_dir()?;
    let mut stats = InheritStats::default();
    let skills_src = global.join("skills");
    let skills_dst = env_dir.join("skills");
    fs::create_dir_all(&skills_dst)?;

    if skills_src.is_dir() {
        for entry in fs::read_dir(&skills_src)
            .with_context(|| format!("failed to read {}", skills_src.display()))?
        {
            let entry = entry?;
            let source = entry.path();
            if !source.is_dir() {
                continue;
            }
            let dest = skills_dst.join(entry.file_name());
            if dest.exists() {
                continue;
            }
            match symlink_dir_or_copy(&source, &dest)? {
                LinkKind::Symlink => stats.skills_linked += 1,
                LinkKind::Copy => stats.skills_copied += 1,
            }
        }
    }

    let settings_src = global.join("settings.json");
    let settings_dst = env_dir.join("settings.json");
    if settings_src.is_file() && !settings_dst.exists() {
        fs::copy(&settings_src, &settings_dst).with_context(|| {
            format!(
                "failed to copy {} to {}",
                settings_src.display(),
                settings_dst.display()
            )
        })?;
        stats.settings_copied = true;
    }

    Ok(stats)
}

enum LinkKind {
    Symlink,
    Copy,
}

fn symlink_dir_or_copy(source: &Path, dest: &Path) -> Result<LinkKind> {
    if create_dir_symlink(source, dest).is_ok() {
        return Ok(LinkKind::Symlink);
    }
    copy_dir_recursive(source, dest)?;
    Ok(LinkKind::Copy)
}

#[cfg(unix)]
fn create_dir_symlink(source: &Path, dest: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, dest)
}

#[cfg(windows)]
fn create_dir_symlink(source: &Path, dest: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(source, dest)
}

fn copy_dir_recursive(source: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest).with_context(|| format!("failed to create {}", dest.display()))?;
    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry?;
        let entry_source = entry.path();
        let entry_dest = dest.join(entry.file_name());
        if entry_source.is_dir() {
            copy_dir_recursive(&entry_source, &entry_dest)?;
        } else {
            fs::copy(&entry_source, &entry_dest).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    entry_source.display(),
                    entry_dest.display()
                )
            })?;
        }
    }
    Ok(())
}

fn ensure_env_bin(dir: &Path) -> Result<()> {
    write_env_shims(dir)
}

/// Copies `CREDENTIALS_FILE` from `source_claude_dir` into `env_dir`, if it
/// exists there. Returns whether a copy happened.
fn copy_credentials(source_claude_dir: &Path, env_dir: &Path) -> Result<bool> {
    let source = source_claude_dir.join(CREDENTIALS_FILE);
    if !source.is_file() {
        return Ok(false);
    }
    let dest = env_dir.join(CREDENTIALS_FILE);
    fs::copy(&source, &dest)
        .with_context(|| format!("failed to copy {} to {}", source.display(), dest.display()))?;
    Ok(true)
}

pub fn ensure_env_exists(name: &str) -> Result<PathBuf> {
    let dir = env_dir(name)?;
    if !dir.is_dir() {
        bail!("environment '{name}' does not exist (create it with `cvm create {name}`)");
    }
    Ok(dir)
}

pub fn list_envs() -> Result<Vec<String>> {
    let dir = envs_dir()?;
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

pub fn remove_env(name: &str) -> Result<()> {
    let dir = ensure_env_exists(name)?;
    let hooks_dir = hooks::hooks_dir()?;
    hooks::run_pre_hook(&hooks_dir, "pre-remove", name, &dir)?;
    fs::remove_dir_all(&dir).with_context(|| format!("failed to remove {}", dir.display()))?;
    hooks::run_post_hook(&hooks_dir, "post-remove", name, &dir);
    Ok(())
}

/// KEY=VALUE pairs the shell hook should `export` to activate `name`,
/// including any variables from that environment's `.env` file.
/// `CLAUDE_CONFIG_DIR`/`CVM_ENV` are appended last so they always win over
/// anything the `.env` file happens to define under the same name.
pub fn resolve_activate(name: &str) -> Result<Vec<(String, String)>> {
    let dir = ensure_env_exists(name)?;
    let hooks_dir = hooks::hooks_dir()?;
    hooks::run_pre_hook(&hooks_dir, "pre-activate", name, &dir)?;
    ensure_env_bin(&dir)?;
    let dir_str = dir
        .to_str()
        .with_context(|| format!("environment path is not valid UTF-8: {}", dir.display()))?
        .to_string();
    let mut pairs = load_env_file(&dir)?;
    pairs.push((CONFIG_DIR_VAR.to_string(), dir_str));
    pairs.push((ACTIVE_ENV_VAR.to_string(), name.to_string()));
    hooks::run_post_hook(&hooks_dir, "post-activate", name, &dir);
    Ok(pairs)
}

/// Variable names the shell hook should unset to deactivate, including any
/// variables loaded from the currently active environment's `.env` file.
pub fn resolve_deactivate() -> Result<Vec<String>> {
    let mut vars = vec![CONFIG_DIR_VAR.to_string(), ACTIVE_ENV_VAR.to_string()];
    if let Some(name) = active_env() {
        if let Ok(dir) = env_dir(&name) {
            let hooks_dir = hooks::hooks_dir()?;
            hooks::run_pre_hook(&hooks_dir, "pre-deactivate", &name, &dir)?;
            if let Ok(env_vars) = load_env_file(&dir) {
                vars.extend(env_vars.into_iter().map(|(key, _)| key));
            }
            hooks::run_post_hook(&hooks_dir, "post-deactivate", &name, &dir);
        }
    }
    Ok(vars)
}

/// Runs `command` as a child process with `CLAUDE_CONFIG_DIR`/`CVM_ENV`, plus
/// any variables from `name`'s `.env` file, scoped to that one process -
/// never touching the parent shell's environment.
pub fn run_in_env(name: &str, command: &[String]) -> Result<i32> {
    let dir = ensure_env_exists(name)?;
    ensure_env_bin(&dir)?;
    let Some((program, args)) = command.split_first() else {
        bail!("no command given to run");
    };
    let env_vars = load_env_file(&dir)?;
    let mut paths = vec![env_bin_dir(&dir)];
    paths.extend(env::split_paths(&env::var_os("PATH").unwrap_or_default()));
    let path = env::join_paths(paths).context("failed to prepend environment bin to PATH")?;
    let mut cmd = Command::new(program);
    cmd.args(args)
        .envs(env_vars)
        .env(CONFIG_DIR_VAR, &dir)
        .env(ACTIVE_ENV_VAR, name)
        .env("PATH", path);
    let status: ExitStatus = cmd
        .status()
        .with_context(|| format!("failed to execute '{program}'"))?;
    Ok(status.code().unwrap_or(1))
}

/// Loads `KEY=VALUE` pairs from an environment directory's `.env` file.
/// Returns an empty list if the file doesn't exist. Blank lines and lines
/// starting with `#` are skipped; malformed lines are skipped with a
/// warning on stderr rather than failing the whole load (this is parsed on
/// every `use`/`run`/`open`, so a typo in one line shouldn't break all of
/// them).
pub fn load_env_file(dir: &Path) -> Result<Vec<(String, String)>> {
    let path = dir.join(DOTENV_FILE);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;

    let mut pairs = Vec::new();
    for (lineno, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            eprintln!(
                "warning: {}:{}: ignoring malformed line (expected KEY=VALUE)",
                path.display(),
                lineno + 1
            );
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            eprintln!(
                "warning: {}:{}: ignoring line with empty key",
                path.display(),
                lineno + 1
            );
            continue;
        }
        if is_reserved_env_key(key) {
            continue;
        }
        pairs.push((key.to_string(), unquote(value.trim()).to_string()));
    }
    Ok(pairs)
}

fn is_reserved_env_key(key: &str) -> bool {
    key.eq_ignore_ascii_case("PATH")
        || key.eq_ignore_ascii_case("PS1")
        || key.eq_ignore_ascii_case(CONFIG_DIR_VAR)
        || key
            .as_bytes()
            .get(..4)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"CVM_"))
}

/// Header comment written at the top of a fresh `.env` file, whether created
/// by `write_env_file` or by `ensure_dotenv_file` for `cvm edit`.
const DOTENV_HEADER: &str =
    "# Local environment variables for this cvm environment (e.g. MCP server credentials).\n\
     # Loaded into the process on `cvm use`/`activate`, `cvm run`, and `cvm open`.\n\
     # Values are NEVER included in `cvm export` - only the variable names are shared.\n";

/// Writes `vars` back to an environment directory's `.env` file, sorted by
/// key for a stable diff.
pub fn write_env_file(dir: &Path, vars: &BTreeMap<String, String>) -> Result<()> {
    let path = dir.join(DOTENV_FILE);
    let mut out = String::from(DOTENV_HEADER);
    for (key, value) in vars {
        out.push_str(key);
        out.push('=');
        out.push_str(value);
        out.push('\n');
    }
    fs::write(&path, out).with_context(|| format!("failed to write {}", path.display()))
}

fn unquote(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.len() >= 2 {
        let (first, last) = (bytes[0], bytes[bytes.len() - 1]);
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return &value[1..value.len() - 1];
        }
    }
    value
}

/// Launches `claude` scoped to `name`, with `CVM_ENV` set to `name` for that
/// process only. Equivalent to `cvm run <name> -- claude`, but as its own
/// command so multiple isolated Claude Code instances can be opened side by
/// side (e.g. one per client/project) without ever touching the parent
/// shell's environment or each other.
pub fn open_env(name: &str) -> Result<i32> {
    run_in_env(name, &["claude".to_string()])
}

/// Ensures `dir`'s `.env` file exists, creating it with the standard header
/// (and no variables) if it doesn't. Returns its path either way.
fn ensure_dotenv_file(dir: &Path) -> Result<PathBuf> {
    let path = dir.join(DOTENV_FILE);
    if !path.is_file() {
        fs::write(&path, DOTENV_HEADER)
            .with_context(|| format!("failed to create {}", path.display()))?;
    }
    Ok(path)
}

/// Opens `name`'s `.env` file in the user's editor (`$VISUAL`, falling back
/// to `$EDITOR`), creating an empty one first if it doesn't exist yet.
/// Returns the editor process's exit code.
pub fn edit_env(name: &str) -> Result<i32> {
    let dir = ensure_env_exists(name)?;
    let path = ensure_dotenv_file(&dir)?;

    let editor = env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .context("no editor configured; set $EDITOR or $VISUAL to edit .env files")?;
    let mut parts = editor.split_whitespace();
    let program = parts.next().context("$EDITOR/$VISUAL is set but empty")?;

    let status: ExitStatus = Command::new(program)
        .args(parts)
        .arg(&path)
        .status()
        .with_context(|| format!("failed to launch editor '{editor}'"))?;
    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serializes tests that temporarily override cvm's home directory.
    static HOME_LOCK: Mutex<()> = Mutex::new(());

    fn with_temp_home<F: FnOnce(&Path)>(f: F) {
        let _guard = HOME_LOCK.lock().unwrap();
        let home = tempfile::tempdir().unwrap();
        let key = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
        let prev = env::var(key).ok();
        let prev_cvm_home = env::var("CVM_HOME").ok();
        let prev_cvm_user_home = env::var("CVM_USER_HOME").ok();
        // SAFETY: guarded by HOME_LOCK so no other test reads home paths concurrently.
        unsafe {
            env::set_var(key, home.path());
            env::set_var("CVM_USER_HOME", home.path());
            env::set_var("CVM_HOME", home.path().join(".cvm"));
        }
        f(home.path());
        match prev {
            Some(v) => unsafe { env::set_var(key, v) },
            None => unsafe { env::remove_var(key) },
        }
        match prev_cvm_home {
            Some(v) => unsafe { env::set_var("CVM_HOME", v) },
            None => unsafe { env::remove_var("CVM_HOME") },
        }
        match prev_cvm_user_home {
            Some(v) => unsafe { env::set_var("CVM_USER_HOME", v) },
            None => unsafe { env::remove_var("CVM_USER_HOME") },
        }
    }

    #[test]
    fn rejects_path_traversal_names() {
        assert!(validate_name("../escape").is_err());
        assert!(validate_name("nested/dir").is_err());
        assert!(validate_name("").is_err());
        assert!(validate_name(".").is_err());
        assert!(validate_name("..").is_err());
        assert!(validate_name("work").is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn resolve_activate_runs_pre_and_post_activate_hooks() {
        with_temp_home(|home| {
            let _exec_guard = hooks::EXEC_TEST_LOCK.lock().unwrap();
            let hooks_dir = home.join(".cvm").join("hooks");
            fs::create_dir_all(&hooks_dir).unwrap();
            let marker = home.join("hook-log.txt");

            for event in ["pre-activate", "post-activate"] {
                let hook = hooks_dir.join(event);
                fs::write(
                    &hook,
                    format!(
                        "#!/bin/sh\necho \"{event} $CVM_ENV\" >> {}\n",
                        marker.display()
                    ),
                )
                .unwrap();
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
            }

            create_env("work", true, false).unwrap();
            resolve_activate("work").unwrap();

            let contents = fs::read_to_string(&marker).unwrap();
            assert_eq!(contents, "pre-activate work\npost-activate work\n");
        });
    }

    #[cfg(unix)]
    #[test]
    fn resolve_activate_aborts_when_pre_activate_hook_fails() {
        with_temp_home(|home| {
            let _exec_guard = hooks::EXEC_TEST_LOCK.lock().unwrap();
            let hooks_dir = home.join(".cvm").join("hooks");
            fs::create_dir_all(&hooks_dir).unwrap();
            let hook = hooks_dir.join("pre-activate");
            fs::write(&hook, "#!/bin/sh\nexit 1\n").unwrap();
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();

            create_env("work", true, false).unwrap();

            assert!(resolve_activate("work").is_err());
        });
    }

    #[cfg(unix)]
    #[test]
    fn resolve_deactivate_runs_pre_and_post_deactivate_hooks_when_env_active() {
        with_temp_home(|home| {
            let _exec_guard = hooks::EXEC_TEST_LOCK.lock().unwrap();
            let hooks_dir = home.join(".cvm").join("hooks");
            fs::create_dir_all(&hooks_dir).unwrap();
            let marker = home.join("hook-log.txt");

            for event in ["pre-deactivate", "post-deactivate"] {
                let hook = hooks_dir.join(event);
                fs::write(
                    &hook,
                    format!(
                        "#!/bin/sh\necho \"{event} $CVM_ENV\" >> {}\n",
                        marker.display()
                    ),
                )
                .unwrap();
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
            }

            create_env("work", true, false).unwrap();
            // SAFETY: guarded by HOME_LOCK (held for the whole with_temp_home closure).
            unsafe {
                env::set_var(ACTIVE_ENV_VAR, "work");
            }

            resolve_deactivate().unwrap();

            let contents = fs::read_to_string(&marker).unwrap();
            assert_eq!(contents, "pre-deactivate work\npost-deactivate work\n");

            unsafe {
                env::remove_var(ACTIVE_ENV_VAR);
            }
        });
    }

    #[test]
    fn resolve_deactivate_skips_hooks_when_no_env_active() {
        with_temp_home(|_home| {
            // No CVM_ENV set - resolve_deactivate must not error and must not
            // try to resolve any hook.
            let vars = resolve_deactivate().unwrap();
            assert!(vars.contains(&CONFIG_DIR_VAR.to_string()));
        });
    }

    #[test]
    fn deactivate_unsets_both_vars() {
        let vars = resolve_deactivate().unwrap();
        assert!(vars.contains(&CONFIG_DIR_VAR.to_string()));
        assert!(vars.contains(&ACTIVE_ENV_VAR.to_string()));
    }

    #[test]
    fn copies_credentials_when_present_in_source() {
        let source = tempfile::tempdir().unwrap();
        let dest = tempfile::tempdir().unwrap();
        fs::write(source.path().join(CREDENTIALS_FILE), "{\"token\":\"abc\"}").unwrap();

        let copied = copy_credentials(source.path(), dest.path()).unwrap();

        assert!(copied);
        assert_eq!(
            fs::read_to_string(dest.path().join(CREDENTIALS_FILE)).unwrap(),
            "{\"token\":\"abc\"}"
        );
    }

    #[test]
    fn skips_copying_when_no_global_credentials_exist() {
        let source = tempfile::tempdir().unwrap();
        let dest = tempfile::tempdir().unwrap();

        let copied = copy_credentials(source.path(), dest.path()).unwrap();

        assert!(!copied);
        assert!(!dest.path().join(CREDENTIALS_FILE).exists());
    }

    #[test]
    fn missing_dotenv_file_loads_as_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load_env_file(dir.path()).unwrap(), Vec::new());
    }

    #[test]
    fn dotenv_parsing_skips_comments_blanks_and_malformed_lines() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(".env"),
            "# a comment\n\nPOSTGRES_PASSWORD=s3cr3t\nQUOTED=\"hello world\"\nnot-a-valid-line\n=novalue\n",
        )
        .unwrap();

        let pairs = load_env_file(dir.path()).unwrap();
        assert_eq!(
            pairs,
            vec![
                ("POSTGRES_PASSWORD".to_string(), "s3cr3t".to_string()),
                ("QUOTED".to_string(), "hello world".to_string()),
            ]
        );
    }

    #[test]
    fn dotenv_parsing_ignores_reserved_activation_vars() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(".env"),
            "PATH=poisoned\n\
             CVM_OLD_PATH=poisoned\n\
             CVM_OLD_PS1=poisoned\n\
             CVM_OLD_PROMPT=poisoned\n\
             CVM_HOME=poisoned\n\
             CVM_AUTO=poisoned\n\
             CLAUDE_CONFIG_DIR=poisoned\n\
             CVM_ENV=poisoned\n\
             SAFE_VALUE=kept\n",
        )
        .unwrap();

        assert_eq!(
            load_env_file(dir.path()).unwrap(),
            vec![("SAFE_VALUE".to_string(), "kept".to_string())]
        );
    }

    #[test]
    fn write_env_file_round_trips_through_load_env_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut vars = BTreeMap::new();
        vars.insert("GITHUB_TOKEN".to_string(), "ghp_example".to_string());
        write_env_file(dir.path(), &vars).unwrap();

        let loaded: BTreeMap<_, _> = load_env_file(dir.path()).unwrap().into_iter().collect();
        assert_eq!(loaded, vars);
    }

    #[test]
    fn ensure_dotenv_file_creates_empty_file_with_header() {
        let dir = tempfile::tempdir().unwrap();

        let path = ensure_dotenv_file(dir.path()).unwrap();

        assert_eq!(path, dir.path().join(DOTENV_FILE));
        assert_eq!(fs::read_to_string(&path).unwrap(), DOTENV_HEADER);
        assert_eq!(load_env_file(dir.path()).unwrap(), Vec::new());
    }

    #[test]
    fn ensure_env_layout_creates_skills_bin_dotenv_and_shims() {
        let dir = tempfile::tempdir().unwrap();
        ensure_env_layout(dir.path()).unwrap();
        assert!(dir.path().join("skills").is_dir());
        assert!(dir.path().join("bin").is_dir());
        assert!(dir.path().join(".env").is_file());
        assert!(dir.path().join("bin/claude").is_file());
        assert!(dir.path().join("bin/skills").is_file());
        assert!(dir.path().join("bin/claude.cmd").is_file());
        assert!(dir.path().join("bin/skills.cmd").is_file());
    }

    #[test]
    fn create_env_creates_skills_bin_and_dotenv() {
        with_temp_home(|home| {
            let (dir, _, _) = create_env("work", true, false).unwrap();
            assert!(dir.starts_with(home.join(".cvm")));
            assert!(dir.join("skills").is_dir());
            assert!(dir.join("bin").is_dir());
            assert!(dir.join(".env").is_file());
        });
    }

    #[cfg(unix)]
    #[test]
    fn create_env_runs_post_create_hook() {
        with_temp_home(|home| {
            // HOME_LOCK (held by with_temp_home) first, then EXEC_TEST_LOCK -
            // the same order everywhere, so the two can never deadlock.
            let _exec_guard = crate::hooks::EXEC_TEST_LOCK.lock().unwrap();
            let hooks_dir = home.join(".cvm").join("hooks");
            fs::create_dir_all(&hooks_dir).unwrap();
            let hook = hooks_dir.join("post-create");
            let marker = home.join("hook-ran.txt");
            fs::write(
                &hook,
                format!(
                    "#!/bin/sh\necho \"$CVM_HOOK_EVENT $CVM_ENV\" > {}\n",
                    marker.display()
                ),
            )
            .unwrap();
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();

            create_env("work", true, false).unwrap();

            let contents = fs::read_to_string(&marker).unwrap();
            assert_eq!(contents.trim(), "post-create work");
        });
    }

    #[test]
    fn create_env_inherits_global_skills_and_settings_when_requested() {
        with_temp_home(|home| {
            let global = global_claude_dir().unwrap();
            assert!(global.starts_with(home));
            let skill = global.join("skills/example");
            fs::create_dir_all(&skill).unwrap();
            fs::write(skill.join("SKILL.md"), "# Example\n").unwrap();
            fs::write(global.join("settings.json"), "{\"theme\":\"dark\"}").unwrap();

            let (dir, _, stats) = create_env("work", true, true).unwrap();
            assert!(dir.starts_with(home.join(".cvm")));

            assert_eq!(stats.skills_linked + stats.skills_copied, 1);
            assert!(stats.settings_copied);
            assert_eq!(
                fs::read_to_string(dir.join("skills/example/SKILL.md")).unwrap(),
                "# Example\n"
            );
            assert_eq!(
                fs::read_to_string(dir.join("settings.json")).unwrap(),
                "{\"theme\":\"dark\"}"
            );
        });
    }

    #[test]
    fn ensure_dotenv_file_leaves_existing_file_untouched() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(DOTENV_FILE), "GITHUB_TOKEN=ghp_example\n").unwrap();

        let path = ensure_dotenv_file(dir.path()).unwrap();

        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "GITHUB_TOKEN=ghp_example\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn remove_env_runs_pre_and_post_remove_hooks() {
        with_temp_home(|home| {
            let _exec_guard = crate::hooks::EXEC_TEST_LOCK.lock().unwrap();
            let hooks_dir = home.join(".cvm").join("hooks");
            fs::create_dir_all(&hooks_dir).unwrap();
            let marker = home.join("hook-log.txt");

            let pre_hook = hooks_dir.join("pre-remove");
            fs::write(
                &pre_hook,
                format!("#!/bin/sh\necho \"pre $CVM_ENV\" >> {}\n", marker.display()),
            )
            .unwrap();
            let post_hook = hooks_dir.join("post-remove");
            fs::write(
                &post_hook,
                format!(
                    "#!/bin/sh\necho \"post $CVM_ENV\" >> {}\n",
                    marker.display()
                ),
            )
            .unwrap();
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&pre_hook, fs::Permissions::from_mode(0o755)).unwrap();
            fs::set_permissions(&post_hook, fs::Permissions::from_mode(0o755)).unwrap();

            create_env("work", true, false).unwrap();
            remove_env("work").unwrap();

            let contents = fs::read_to_string(&marker).unwrap();
            assert_eq!(contents, "pre work\npost work\n");
        });
    }

    #[cfg(unix)]
    #[test]
    fn remove_env_aborts_when_pre_remove_hook_fails() {
        with_temp_home(|home| {
            let _exec_guard = crate::hooks::EXEC_TEST_LOCK.lock().unwrap();
            let hooks_dir = home.join(".cvm").join("hooks");
            fs::create_dir_all(&hooks_dir).unwrap();
            let pre_hook = hooks_dir.join("pre-remove");
            fs::write(&pre_hook, "#!/bin/sh\nexit 1\n").unwrap();
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&pre_hook, fs::Permissions::from_mode(0o755)).unwrap();

            let (dir, _, _) = create_env("work", true, false).unwrap();

            assert!(remove_env("work").is_err());
            assert!(
                dir.is_dir(),
                "environment must not be deleted when pre-remove fails"
            );
        });
    }
}
