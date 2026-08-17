//! Self-update: checks GitHub Releases for a newer `cvm` and, if found,
//! downloads the right asset for this platform and swaps the running
//! binary in place via the `self-replace` crate.
//!
//! Deliberately shells out to `curl` and `tar` instead of adding an HTTP
//! client crate - the release profile optimizes hard for binary size, and
//! `install.sh` already requires `curl` on the machine, so this adds no new
//! runtime requirement.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

const REPO: &str = "acwoss/cvm";
const BIN_NAME: &str = if cfg!(windows) { "cvm.exe" } else { "cvm" };

pub enum UpdateOutcome {
    AlreadyUpToDate { version: String },
    Updated { from: String, to: String },
}

/// The current build's version, as set by Cargo from `Cargo.toml`.
pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The release target triple this binary was built for, matching one of
/// the targets built by `.github/workflows/release.yml`.
fn target_triple() -> Result<&'static str> {
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Ok("x86_64-unknown-linux-gnu")
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        Ok("aarch64-unknown-linux-gnu")
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Ok("x86_64-apple-darwin")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Ok("aarch64-apple-darwin")
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Ok("x86_64-pc-windows-msvc")
    } else {
        bail!(
            "no prebuilt release for this platform; install manually via `cargo install --path .`"
        )
    }
}

/// The archive filename `release.yml` publishes for `target`.
fn asset_name(target: &str) -> String {
    let ext = if target.contains("windows") {
        "zip"
    } else {
        "tar.gz"
    };
    format!("cvm-{target}.{ext}")
}

/// Latest release tag (e.g. `"v0.2.0"`) from the GitHub API, fetched via
/// `curl` and parsed with `serde_json` (both already dependencies/tools we
/// need elsewhere, so no HTTP client crate is added just for this).
pub(crate) fn fetch_latest_tag() -> Result<String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let output = Command::new("curl")
        .args(["-fsSL", "-H", "User-Agent: cvm-updater"])
        .arg(&url)
        .output()
        .context("failed to run 'curl' - is it installed and on PATH?")?;
    if !output.status.success() {
        bail!(
            "failed to fetch latest release info from {url}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let body: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("failed to parse GitHub release JSON")?;
    let tag = body["tag_name"]
        .as_str()
        .context("GitHub release response had no 'tag_name'")?;
    Ok(tag.to_string())
}

fn download(url: &str, dest: &Path) -> Result<()> {
    let status = Command::new("curl")
        .args(["-fsSL", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .context("failed to run 'curl' - is it installed and on PATH?")?;
    if !status.success() {
        bail!("failed to download {url}");
    }
    Ok(())
}

/// Extracts `archive` (`.tar.gz` or `.zip`) into `dest_dir` via `tar` -
/// available as GNU tar on Linux and as bsdtar (which also reads `.zip`)
/// on macOS and Windows.
fn extract(archive: &Path, dest_dir: &Path) -> Result<()> {
    let flag = if archive.extension().and_then(|e| e.to_str()) == Some("zip") {
        "-xf"
    } else {
        "-xzf"
    };
    let status = Command::new("tar")
        .arg(flag)
        .arg(archive)
        .arg("-C")
        .arg(dest_dir)
        .status()
        .context("failed to run 'tar' - is it installed and on PATH?")?;
    if !status.success() {
        bail!("failed to extract {}", archive.display());
    }
    Ok(())
}

fn find_binary(dir: &Path) -> Result<PathBuf> {
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Ok(found) = find_binary(&path) {
                return Ok(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(BIN_NAME) {
            return Ok(path);
        }
    }
    bail!(
        "could not find '{BIN_NAME}' inside the downloaded archive at {}",
        dir.display()
    );
}

/// Checks for a newer release and, if one exists, downloads it and replaces
/// the currently running binary.
pub fn run() -> Result<UpdateOutcome> {
    let current = current_version();
    let latest_tag = fetch_latest_tag()?;
    let latest = latest_tag.trim_start_matches('v');

    if latest == current {
        return Ok(UpdateOutcome::AlreadyUpToDate {
            version: current.to_string(),
        });
    }

    let target = target_triple()?;
    let asset = asset_name(target);
    let url = format!("https://github.com/{REPO}/releases/download/{latest_tag}/{asset}");

    let tmp_dir = std::env::temp_dir().join(format!("cvm-update-{}", std::process::id()));
    fs::create_dir_all(&tmp_dir)
        .with_context(|| format!("failed to create {}", tmp_dir.display()))?;
    let cleanup = |dir: &Path| {
        let _ = fs::remove_dir_all(dir);
    };

    let archive_path = tmp_dir.join(&asset);
    if let Err(err) = download(&url, &archive_path) {
        cleanup(&tmp_dir);
        return Err(err);
    }
    if let Err(err) = extract(&archive_path, &tmp_dir) {
        cleanup(&tmp_dir);
        return Err(err);
    }
    let new_binary = match find_binary(&tmp_dir) {
        Ok(path) => path,
        Err(err) => {
            cleanup(&tmp_dir);
            return Err(err);
        }
    };

    let result = self_replace::self_replace(&new_binary).context("failed to replace binary");
    cleanup(&tmp_dir);
    result?;

    Ok(UpdateOutcome::Updated {
        from: current.to_string(),
        to: latest.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_name_uses_zip_only_for_windows() {
        assert_eq!(
            asset_name("x86_64-pc-windows-msvc"),
            "cvm-x86_64-pc-windows-msvc.zip"
        );
        assert_eq!(
            asset_name("x86_64-unknown-linux-gnu"),
            "cvm-x86_64-unknown-linux-gnu.tar.gz"
        );
        assert_eq!(
            asset_name("aarch64-apple-darwin"),
            "cvm-aarch64-apple-darwin.tar.gz"
        );
    }

    #[test]
    fn target_triple_resolves_to_a_known_release_target() {
        let target = target_triple().expect("this platform should be a supported release target");
        assert!([
            "x86_64-unknown-linux-gnu",
            "aarch64-unknown-linux-gnu",
            "x86_64-apple-darwin",
            "aarch64-apple-darwin",
            "x86_64-pc-windows-msvc"
        ]
        .contains(&target));
    }

    #[test]
    fn find_binary_locates_nested_binary_and_ignores_other_files() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("cvm-x86_64-unknown-linux-gnu");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("README.md"), "not the binary").unwrap();
        fs::write(nested.join(BIN_NAME), "fake binary contents").unwrap();

        let found = find_binary(dir.path()).unwrap();

        assert_eq!(found, nested.join(BIN_NAME));
    }

    #[test]
    fn find_binary_errors_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(find_binary(dir.path()).is_err());
    }
}
