use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub fn env_bin_dir(env_dir: &Path) -> PathBuf {
    env_dir.join("bin")
}

pub fn write_env_shims(env_dir: &Path) -> Result<()> {
    let bin_dir = env_bin_dir(env_dir);
    fs::create_dir_all(&bin_dir)
        .with_context(|| format!("failed to create {}", bin_dir.display()))?;

    write_shim(&bin_dir.join("claude"), &unix_shim("exec claude \"$@\""))?;
    write_shim(
        &bin_dir.join("skills"),
        &unix_shim("exec npx --yes skills \"$@\""),
    )?;
    write_shim(&bin_dir.join("claude.cmd"), &windows_shim("claude %*"))?;
    write_shim(
        &bin_dir.join("skills.cmd"),
        &windows_shim("npx --yes skills %*"),
    )?;

    Ok(())
}

fn write_shim(path: &Path, contents: &str) -> Result<()> {
    fs::write(path, contents).with_context(|| format!("failed to write {}", path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("failed to make {} executable", path.display()))?;
    }

    Ok(())
}

fn unix_shim(exec_line: &str) -> String {
    format!(
        r#"#!/bin/sh
DIR=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
export CLAUDE_CONFIG_DIR="$DIR"
export CVM_ENV=$(basename "$DIR")
BIN_DIR="$DIR/bin"
OLD_IFS=$IFS
IFS=:
NEW_PATH=
for p in $PATH; do
  [ "$p" = "$BIN_DIR" ] && continue
  NEW_PATH="${{NEW_PATH:+$NEW_PATH:}}$p"
done
IFS=$OLD_IFS
export PATH="$NEW_PATH"
hash -r 2>/dev/null || true
{exec_line}
"#
    )
}

fn windows_shim(exec_line: &str) -> String {
    format!(
        r#"@echo off
set "DIR=%~dp0.."
for %%I in ("%DIR%") do set "DIR=%%~fI"
set "CLAUDE_CONFIG_DIR=%DIR%"
for %%I in ("%DIR%") do set "CVM_ENV=%%~nxI"
call :cvm_strip_bin
{exec_line}
exit /b %ERRORLEVEL%

:cvm_strip_bin
set "CVM_SHIM_BIN=%~dp0"
set "CVM_SHIM_OLD_PATH=%PATH%"
set "PATH="
for %%P in ("%CVM_SHIM_OLD_PATH:;=" "%") do call :cvm_append_path "%%~P"
set "CVM_SHIM_BIN="
set "CVM_SHIM_OLD_PATH="
exit /b

:cvm_append_path
for %%I in ("%~1") do if /I "%%~fI\"=="%CVM_SHIM_BIN%" exit /b
if defined PATH (set "PATH=%PATH%;%~1") else set "PATH=%~1"
exit /b
"#
    )
}
