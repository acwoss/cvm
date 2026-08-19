//! Self-update: checks GitHub Releases for a newer `cvm` and, if found,
//! downloads the right asset for this platform and swaps the running
//! binary in place via `cvm_core::update`. See that module for the shared
//! fetch/download/extract/replace primitives - also used by `cvm-ui`'s
//! own self-update and by `cvm launch` to fetch `cvm-ui`.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use cvm_core::update as shared;

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

pub fn fetch_latest_tag() -> Result<String> {
    shared::fetch_latest_tag(REPO, 10)
}

/// Checks for a newer release and, if one exists, downloads it and replaces
/// the currently running binary.
pub fn run() -> Result<UpdateOutcome> {
    let current = current_version();
    let latest_tag = fetch_latest_tag()?;
    let latest = latest_tag.trim_start_matches('v');

    // Only ever move forward - if the "latest" release is the same as or
    // older than what's running (e.g. a stale/out-of-order GitHub release),
    // treat it as already up to date instead of "updating" to an older
    // version.
    let is_newer = match (semver::Version::parse(latest), semver::Version::parse(current)) {
        (Ok(latest_ver), Ok(current_ver)) => latest_ver > current_ver,
        _ => latest != current,
    };
    if !is_newer {
        return Ok(UpdateOutcome::AlreadyUpToDate {
            version: current.to_string(),
        });
    }

    let target = shared::target_triple()?;
    let asset = shared::asset_name("cvm", target);
    let url = format!("https://github.com/{REPO}/releases/download/{latest_tag}/{asset}");

    let tmp_dir = std::env::temp_dir().join(format!("cvm-update-{}", std::process::id()));
    fs::create_dir_all(&tmp_dir)
        .with_context(|| format!("failed to create {}", tmp_dir.display()))?;
    let cleanup = |dir: &Path| {
        let _ = fs::remove_dir_all(dir);
    };

    let archive_path = tmp_dir.join(&asset);
    if let Err(err) = shared::download_asset(&url, &archive_path) {
        cleanup(&tmp_dir);
        return Err(err);
    }
    if let Err(err) = shared::extract_asset(&archive_path, &tmp_dir) {
        cleanup(&tmp_dir);
        return Err(err);
    }
    let new_binary = match shared::find_binary(&tmp_dir, BIN_NAME) {
        Ok(path) => path,
        Err(err) => {
            cleanup(&tmp_dir);
            return Err(err);
        }
    };

    let result = shared::replace_running_binary(&new_binary);
    cleanup(&tmp_dir);
    result?;

    Ok(UpdateOutcome::Updated {
        from: current.to_string(),
        to: latest.to_string(),
    })
}
