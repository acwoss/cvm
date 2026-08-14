use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

use anyhow::{bail, Context, Result};

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

pub fn cvm_home() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine the home directory")?;
    Ok(home.join(".cvm"))
}

pub fn envs_dir() -> Result<PathBuf> {
    Ok(cvm_home()?.join("envs"))
}

/// The global Claude Code config directory (`~/.claude`), regardless of any
/// `CLAUDE_CONFIG_DIR` override currently in effect - this is where a new
/// environment's credentials are copied from, not wherever the active
/// environment happens to point.
fn global_claude_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine the home directory")?;
    Ok(home.join(".claude"))
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
/// environment directory and whether credentials were copied.
pub fn create_env(name: &str, anonymous: bool) -> Result<(PathBuf, bool)> {
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

    Ok((dir, credentials_copied))
}

/// Ensures an environment directory has the standard layout: `skills/`, `bin/`,
/// and a starter `.env` file.
pub fn ensure_env_layout(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir.join("skills"))?;
    fs::create_dir_all(dir.join("bin"))?;
    let _ = ensure_dotenv_file(dir)?;
    Ok(())
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
    fs::remove_dir_all(&dir).with_context(|| format!("failed to remove {}", dir.display()))?;
    Ok(())
}

/// KEY=VALUE pairs the shell hook should `export` to activate `name`,
/// including any variables from that environment's `.env` file.
/// `CLAUDE_CONFIG_DIR`/`CVM_ENV` are appended last so they always win over
/// anything the `.env` file happens to define under the same name.
pub fn resolve_activate(name: &str) -> Result<Vec<(String, String)>> {
    let dir = ensure_env_exists(name)?;
    let dir_str = dir
        .to_str()
        .with_context(|| format!("environment path is not valid UTF-8: {}", dir.display()))?
        .to_string();
    let mut pairs = load_env_file(&dir)?;
    pairs.push((CONFIG_DIR_VAR.to_string(), dir_str));
    pairs.push((ACTIVE_ENV_VAR.to_string(), name.to_string()));
    Ok(pairs)
}

/// Variable names the shell hook should unset to deactivate, including any
/// variables loaded from the currently active environment's `.env` file.
pub fn resolve_deactivate() -> Vec<String> {
    let mut vars = vec![CONFIG_DIR_VAR.to_string(), ACTIVE_ENV_VAR.to_string()];
    if let Some(name) = active_env() {
        if let Ok(dir) = env_dir(&name) {
            if let Ok(env_vars) = load_env_file(&dir) {
                vars.extend(env_vars.into_iter().map(|(key, _)| key));
            }
        }
    }
    vars
}

/// Runs `command` as a child process with `CLAUDE_CONFIG_DIR`/`CVM_ENV`, plus
/// any variables from `name`'s `.env` file, scoped to that one process -
/// never touching the parent shell's environment.
pub fn run_in_env(name: &str, command: &[String]) -> Result<i32> {
    let dir = ensure_env_exists(name)?;
    let Some((program, args)) = command.split_first() else {
        bail!("no command given to run");
    };
    let env_vars = load_env_file(&dir)?;
    let status: ExitStatus = Command::new(program)
        .args(args)
        .envs(env_vars)
        .env(CONFIG_DIR_VAR, &dir)
        .env(ACTIVE_ENV_VAR, name)
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
        pairs.push((key.to_string(), unquote(value.trim()).to_string()));
    }
    Ok(pairs)
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

    #[test]
    fn rejects_path_traversal_names() {
        assert!(validate_name("../escape").is_err());
        assert!(validate_name("nested/dir").is_err());
        assert!(validate_name("").is_err());
        assert!(validate_name(".").is_err());
        assert!(validate_name("..").is_err());
        assert!(validate_name("work").is_ok());
    }

    #[test]
    fn deactivate_unsets_both_vars() {
        let vars = resolve_deactivate();
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
    fn create_env_creates_skills_bin_and_dotenv() {
        let dir = tempfile::tempdir().unwrap();
        ensure_env_layout(dir.path()).unwrap();
        assert!(dir.path().join("skills").is_dir());
        assert!(dir.path().join("bin").is_dir());
        assert!(dir.path().join(".env").is_file());
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
}
