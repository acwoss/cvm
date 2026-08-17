//! `cvm launch`: resolves the UI binary under `~/.cvm/bin/`, downloading it
//! from the matching GitHub release asset if missing, then spawns it
//! detached. Mirrors the download/extract approach in `update.rs`, but
//! targets a different asset (`cvm-ui-<target>.<ext>`) and never replaces
//! the running `cvm` binary.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context, Result};

const REPO: &str = "acwoss/cvm";
const UI_BIN_NAME: &str = if cfg!(windows) {
    "cvm-ui.exe"
} else {
    "cvm-ui"
};

fn target_triple() -> Result<&'static str> {
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Ok("x86_64-unknown-linux-gnu")
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Ok("x86_64-apple-darwin")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Ok("aarch64-apple-darwin")
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Ok("x86_64-pc-windows-msvc")
    } else {
        bail!(
            "no prebuilt cvm-ui release for this platform yet; \
             build it manually from cvm-ui/ with `pnpm tauri build`"
        )
    }
}

fn ui_bin_dir() -> Result<PathBuf> {
    Ok(cvm_core::env::cvm_home()?.join("bin"))
}

fn ui_bin_path() -> Result<PathBuf> {
    Ok(ui_bin_dir()?.join(UI_BIN_NAME))
}

fn asset_name(target: &str) -> String {
    let ext = if target.contains("windows") {
        "zip"
    } else {
        "tar.gz"
    };
    format!("cvm-ui-{target}.{ext}")
}

/// Ensures the UI binary exists locally, downloading it from the given
/// release tag if not. Returns its path.
pub fn ensure_installed(fetch_latest_tag: impl Fn() -> Result<String>) -> Result<PathBuf> {
    let path = ui_bin_path()?;
    if path.is_file() {
        return Ok(path);
    }

    let tag = fetch_latest_tag()?;
    let target = target_triple()?;
    let asset = asset_name(target);
    let url = format!("https://github.com/{REPO}/releases/download/{tag}/{asset}");

    let bin_dir = ui_bin_dir()?;
    fs::create_dir_all(&bin_dir)
        .with_context(|| format!("failed to create {}", bin_dir.display()))?;
    let tmp_archive = bin_dir.join(&asset);

    let status = Command::new("curl")
        .args(["-fsSL", "-o"])
        .arg(&tmp_archive)
        .arg(&url)
        .status()
        .context("failed to run 'curl' - is it installed and on PATH?")?;
    if !status.success() {
        let _ = fs::remove_file(&tmp_archive);
        bail!("failed to download {url}");
    }

    let extract_flag = if asset.ends_with(".zip") {
        "-xf"
    } else {
        "-xzf"
    };
    let status = Command::new("tar")
        .arg(extract_flag)
        .arg(&tmp_archive)
        .arg("-C")
        .arg(&bin_dir)
        .status()
        .context("failed to run 'tar' - is it installed and on PATH?")?;
    let _ = fs::remove_file(&tmp_archive);
    if !status.success() {
        bail!("failed to extract {asset}");
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("failed to make {} executable", path.display()))?;
    }

    if !path.is_file() {
        bail!(
            "downloaded {asset} but did not find '{UI_BIN_NAME}' inside it at {}",
            path.display()
        );
    }
    Ok(path)
}

/// Spawns the UI binary detached from the current process.
pub fn spawn(path: &std::path::Path) -> Result<()> {
    Command::new(path)
        .spawn()
        .with_context(|| format!("failed to launch {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static HOME_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn asset_name_uses_zip_only_for_windows() {
        assert_eq!(
            asset_name("x86_64-pc-windows-msvc"),
            "cvm-ui-x86_64-pc-windows-msvc.zip"
        );
        assert_eq!(
            asset_name("x86_64-unknown-linux-gnu"),
            "cvm-ui-x86_64-unknown-linux-gnu.tar.gz"
        );
    }

    #[test]
    fn ensure_installed_skips_download_when_binary_already_present() {
        let _guard = HOME_LOCK.lock().unwrap();
        let home = tempfile::tempdir().unwrap();
        // SAFETY: guardado por HOME_LOCK.
        unsafe {
            std::env::set_var("CVM_HOME", home.path().join(".cvm"));
        }
        let bin_dir = home.path().join(".cvm/bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join(UI_BIN_NAME), "fake binary").unwrap();

        let called = std::cell::Cell::new(false);
        let path = ensure_installed(|| {
            called.set(true);
            Ok("v0.0.0".to_string())
        })
        .unwrap();

        assert!(
            !called.get(),
            "must not fetch a release tag when the binary already exists"
        );
        assert_eq!(path, bin_dir.join(UI_BIN_NAME));
        // SAFETY: guardado por HOME_LOCK.
        unsafe {
            std::env::remove_var("CVM_HOME");
        }
    }
}
