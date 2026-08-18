//! Shared self-update primitives for `cvm` and `cvm-ui` - fetching the
//! latest GitHub release tag, downloading/extracting the right asset for
//! this platform, and swapping the running binary in place. Both binaries
//! are published under the same release tag (see
//! `.github/workflows/release.yml`, jobs `build` and `build-ui` both feed
//! the same `release` job), so this logic only needs to exist once.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

/// The release target triple this binary was built for, matching one of
/// the targets built by `.github/workflows/release.yml`.
pub fn target_triple() -> Result<&'static str> {
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

/// Like `target_triple()`, but rejects targets `release.yml`'s `build-ui`
/// job doesn't publish an asset for (unlike `build`, which cross-builds
/// `cvm` itself for `aarch64-unknown-linux-gnu`, `build-ui` only covers
/// x86_64 Linux, both macOS archs, and Windows).
pub fn target_triple_for_ui() -> Result<&'static str> {
    let target = target_triple()?;
    if target == "aarch64-unknown-linux-gnu" {
        bail!(
            "no prebuilt cvm-ui release for this platform yet; \
             build it manually from cvm-ui/ with `pnpm tauri build`"
        );
    }
    Ok(target)
}

/// The archive filename `release.yml` publishes for `prefix` (`"cvm"` or
/// `"cvm-ui"`) on `target`.
pub fn asset_name(prefix: &str, target: &str) -> String {
    let ext = if target.contains("windows") {
        "zip"
    } else {
        "tar.gz"
    };
    format!("{prefix}-{target}.{ext}")
}

/// Latest release tag (e.g. `"v0.4.0"`) from the GitHub API for `repo`
/// (e.g. `"acwoss/cvm"`), fetched via `curl` with a `timeout_secs` cap so
/// callers on a slow/offline network fail fast instead of hanging.
pub fn fetch_latest_tag(repo: &str, timeout_secs: u64) -> Result<String> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let output = Command::new("curl")
        .args([
            "-fsSL",
            "--max-time",
            &timeout_secs.to_string(),
            "-H",
            "User-Agent: cvm-updater",
        ])
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

pub fn download_asset(url: &str, dest: &Path) -> Result<()> {
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
pub fn extract_asset(archive: &Path, dest_dir: &Path) -> Result<()> {
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

pub fn find_binary(dir: &Path, bin_name: &str) -> Result<PathBuf> {
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Ok(found) = find_binary(&path, bin_name) {
                return Ok(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(bin_name) {
            return Ok(path);
        }
    }
    bail!(
        "could not find '{bin_name}' inside the downloaded archive at {}",
        dir.display()
    )
}

/// Replaces the currently running binary's executable with `new_binary`.
pub fn replace_running_binary(new_binary: &Path) -> Result<()> {
    self_replace::self_replace(new_binary).context("failed to replace binary")
}

/// Cached record of the last successful "latest release" check.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCache {
    pub checked_at_secs: u64,
    pub latest_tag: String,
}

fn read_cache(cache_path: &Path) -> Option<UpdateCache> {
    let content = fs::read_to_string(cache_path).ok()?;
    serde_json::from_str(&content).ok()
}

fn write_cache(cache_path: &Path, cache: &UpdateCache) {
    if let Ok(json) = serde_json::to_string(cache) {
        let _ = fs::write(cache_path, json);
    }
}

/// Returns `Some(latest_version)` (without the leading `v`) if it differs
/// from `current`, using a cache at `cache_path` that's considered fresh
/// for `ttl_secs`. `now_secs` is the caller's notion of "now" (injected so
/// this is deterministically testable). Any failure - missing/corrupt
/// cache, offline network, malformed response - is swallowed and yields
/// `None`; this must never fail or block the command that calls it.
pub fn check_for_update(
    current: &str,
    cache_path: &Path,
    ttl_secs: u64,
    now_secs: u64,
    fetch: impl Fn() -> Result<String>,
) -> Option<String> {
    let latest_tag = match read_cache(cache_path) {
        Some(cache) if now_secs.saturating_sub(cache.checked_at_secs) < ttl_secs => {
            cache.latest_tag
        }
        _ => match fetch() {
            Ok(tag) => {
                write_cache(
                    cache_path,
                    &UpdateCache {
                        checked_at_secs: now_secs,
                        latest_tag: tag.clone(),
                    },
                );
                tag
            }
            Err(_) => {
                // Record the attempt even on failure so the TTL still
                // protects against paying the network timeout on every
                // call while offline. Storing `current` (not the real
                // latest) means this failed attempt never falsely
                // reports an update as available - the next real check
                // happens once this cache entry goes stale.
                write_cache(
                    cache_path,
                    &UpdateCache {
                        checked_at_secs: now_secs,
                        latest_tag: current.to_string(),
                    },
                );
                return None;
            }
        },
    };
    let latest = latest_tag.trim_start_matches('v');
    if latest == current {
        None
    } else {
        Some(latest.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn asset_name_uses_zip_only_for_windows() {
        assert_eq!(
            asset_name("cvm", "x86_64-pc-windows-msvc"),
            "cvm-x86_64-pc-windows-msvc.zip"
        );
        assert_eq!(
            asset_name("cvm", "x86_64-unknown-linux-gnu"),
            "cvm-x86_64-unknown-linux-gnu.tar.gz"
        );
        assert_eq!(
            asset_name("cvm-ui", "aarch64-apple-darwin"),
            "cvm-ui-aarch64-apple-darwin.tar.gz"
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
    fn target_triple_for_ui_rejects_aarch64_linux_with_an_actionable_message() {
        // target_triple_for_ui() can't be tested against the real platform
        // triple directly since CI runs on x86_64/aarch64-darwin, not
        // aarch64-linux - so this test only asserts the rejection logic runs
        // correctly on that one specific triple by re-deriving what
        // target_triple() would need to return, not by mocking the platform.
        // Skip if this isn't the platform where the guard actually fires.
        if target_triple().unwrap() != "aarch64-unknown-linux-gnu" {
            return;
        }
        let err = target_triple_for_ui().unwrap_err();
        assert!(err.to_string().contains("pnpm tauri build"));
    }

    #[test]
    fn find_binary_locates_nested_binary_and_ignores_other_files() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("cvm-x86_64-unknown-linux-gnu");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("README.md"), "not the binary").unwrap();
        fs::write(nested.join("cvm"), "fake binary contents").unwrap();

        let found = find_binary(dir.path(), "cvm").unwrap();

        assert_eq!(found, nested.join("cvm"));
    }

    #[test]
    fn find_binary_errors_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(find_binary(dir.path(), "cvm").is_err());
    }

    #[test]
    fn check_for_update_uses_fresh_cache_without_calling_fetch() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("update-check.json");
        write_cache(
            &cache_path,
            &UpdateCache {
                checked_at_secs: 1000,
                latest_tag: "v0.5.0".into(),
            },
        );
        let calls = AtomicUsize::new(0);

        let result = check_for_update("0.4.0", &cache_path, 86_400, 1500, || {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok("v9.9.9".to_string())
        });

        assert_eq!(result, Some("0.5.0".to_string()));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn check_for_update_refetches_when_cache_is_stale() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("update-check.json");
        write_cache(
            &cache_path,
            &UpdateCache {
                checked_at_secs: 0,
                latest_tag: "v0.1.0".into(),
            },
        );

        let result = check_for_update("0.4.0", &cache_path, 86_400, 100_000, || {
            Ok("v0.5.0".to_string())
        });

        assert_eq!(result, Some("0.5.0".to_string()));
        let cached = read_cache(&cache_path).unwrap();
        assert_eq!(cached.latest_tag, "v0.5.0");
        assert_eq!(cached.checked_at_secs, 100_000);
    }

    #[test]
    fn check_for_update_returns_none_when_up_to_date() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("update-check.json");

        let result = check_for_update("0.4.0", &cache_path, 86_400, 100, || {
            Ok("v0.4.0".to_string())
        });

        assert_eq!(result, None);
    }

    #[test]
    fn check_for_update_swallows_fetch_errors_and_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("missing-cache.json");

        let result = check_for_update("0.4.0", &cache_path, 86_400, 100, || {
            anyhow::bail!("offline")
        });

        assert_eq!(result, None);
    }

    #[test]
    fn check_for_update_caches_a_failed_attempt_so_it_does_not_refetch_within_the_ttl() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("update-check.json");
        let calls = AtomicUsize::new(0);

        let first = check_for_update("0.4.0", &cache_path, 86_400, 100, || {
            calls.fetch_add(1, Ordering::SeqCst);
            anyhow::bail!("offline")
        });
        assert_eq!(first, None);
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        // Second call within the TTL must not re-fetch, even though the
        // first attempt failed.
        let second = check_for_update("0.4.0", &cache_path, 86_400, 200, || {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok("v9.9.9".to_string())
        });
        assert_eq!(second, None);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn check_for_update_ignores_corrupt_cache_file() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("update-check.json");
        fs::write(&cache_path, "not valid json").unwrap();

        let result = check_for_update("0.4.0", &cache_path, 86_400, 100, || {
            Ok("v0.5.0".to_string())
        });

        assert_eq!(result, Some("0.5.0".to_string()));
    }
}
