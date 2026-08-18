//! `cvm launch`: resolves the UI binary under `~/.cvm/bin/`, downloading it
//! from the matching GitHub release asset if missing, then spawns it
//! detached. Mirrors the download/extract approach in `cvm_core::update`,
//! but targets a different asset (`cvm-ui-<target>.<ext>`) and never
//! replaces the running `cvm` binary.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};

const REPO: &str = "acwoss/cvm";
const UI_BIN_NAME: &str = if cfg!(windows) {
    "cvm-ui.exe"
} else {
    "cvm-ui"
};

fn ui_bin_dir() -> Result<PathBuf> {
    Ok(cvm_core::env::cvm_home()?.join("bin"))
}

fn ui_bin_path() -> Result<PathBuf> {
    Ok(ui_bin_dir()?.join(UI_BIN_NAME))
}

/// Ensures the UI binary exists locally, downloading it from the given
/// release tag if not. Returns its path.
pub fn ensure_installed(fetch_latest_tag: impl Fn() -> Result<String>) -> Result<PathBuf> {
    let path = ui_bin_path()?;
    if path.is_file() {
        return Ok(path);
    }

    let tag = fetch_latest_tag()?;
    let target = cvm_core::update::target_triple_for_ui()?;
    let asset = cvm_core::update::asset_name("cvm-ui", target);
    let url = format!("https://github.com/{REPO}/releases/download/{tag}/{asset}");

    let bin_dir = ui_bin_dir()?;
    fs::create_dir_all(&bin_dir)
        .with_context(|| format!("failed to create {}", bin_dir.display()))?;

    let tmp_dir = std::env::temp_dir().join(format!("cvm-launch-{}", std::process::id()));
    fs::create_dir_all(&tmp_dir)
        .with_context(|| format!("failed to create {}", tmp_dir.display()))?;
    let cleanup = |dir: &std::path::Path| {
        let _ = fs::remove_dir_all(dir);
    };

    let tmp_archive = tmp_dir.join(&asset);
    if let Err(err) = cvm_core::update::download_asset(&url, &tmp_archive) {
        cleanup(&tmp_dir);
        return Err(err);
    }
    if let Err(err) = cvm_core::update::extract_asset(&tmp_archive, &tmp_dir) {
        cleanup(&tmp_dir);
        return Err(err);
    }

    let found = match cvm_core::update::find_binary(&tmp_dir, UI_BIN_NAME) {
        Ok(found) => found,
        Err(err) => {
            cleanup(&tmp_dir);
            return Err(err);
        }
    };

    if let Err(err) = fs::rename(&found, &path) {
        if let Err(copy_err) = fs::copy(&found, &path) {
            cleanup(&tmp_dir);
            return Err(copy_err).with_context(|| {
                format!(
                    "failed to install {} to {} (rename also failed: {err})",
                    found.display(),
                    path.display()
                )
            });
        }
    }
    cleanup(&tmp_dir);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("failed to make {} executable", path.display()))?;
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

    #[cfg(unix)]
    #[test]
    fn ensure_installed_finds_binary_nested_inside_a_directory_in_the_archive() {
        let _guard = HOME_LOCK.lock().unwrap();
        let home = tempfile::tempdir().unwrap();
        // SAFETY: guardado por HOME_LOCK.
        unsafe {
            std::env::set_var("CVM_HOME", home.path().join(".cvm"));
        }

        let staging = tempfile::tempdir().unwrap();
        let nested_dir = staging.path().join("cvm-ui-fake-target");
        std::fs::create_dir_all(&nested_dir).unwrap();
        std::fs::write(nested_dir.join(UI_BIN_NAME), "fake ui binary").unwrap();
        let archive_path = staging.path().join("fake.tar.gz");
        let status = std::process::Command::new("tar")
            .arg("-czf")
            .arg(&archive_path)
            .arg("-C")
            .arg(staging.path())
            .arg("cvm-ui-fake-target")
            .status()
            .unwrap();
        assert!(status.success());

        let fake_bin_dir = home.path().join("fake-bin");
        std::fs::create_dir_all(&fake_bin_dir).unwrap();
        let fake_curl = fake_bin_dir.join("curl");
        std::fs::write(
            &fake_curl,
            format!(
                "#!/bin/sh\n# args: -fsSL -o <dest> <url> - just copy our prebuilt archive to <dest>\ncp {} \"$3\"\n",
                archive_path.display()
            ),
        )
        .unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&fake_curl, std::fs::Permissions::from_mode(0o755)).unwrap();

        let old_path = std::env::var("PATH").unwrap_or_default();
        // SAFETY: guardado por HOME_LOCK.
        unsafe {
            std::env::set_var("PATH", format!("{}:{}", fake_bin_dir.display(), old_path));
        }

        let result = ensure_installed(|| Ok("v0.0.0".to_string()));

        // SAFETY: guardado por HOME_LOCK.
        unsafe {
            std::env::set_var("PATH", old_path);
            std::env::remove_var("CVM_HOME");
        }

        let path = result.unwrap();
        assert_eq!(path, home.path().join(".cvm/bin").join(UI_BIN_NAME));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "fake ui binary");
        assert!(!home.path().join(".cvm/bin/cvm-ui-fake-target").exists());
    }
}
