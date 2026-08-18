use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

fn claude_md_path(env_dir: &Path) -> PathBuf {
    env_dir.join("CLAUDE.md")
}

pub fn read_claude_md(env_dir: &Path) -> Result<String> {
    match fs::read_to_string(claude_md_path(env_dir)) {
        Ok(content) => Ok(content),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(err) => Err(err).context("failed to read CLAUDE.md"),
    }
}

pub fn write_claude_md(env_dir: &Path, content: &str) -> Result<()> {
    fs::write(claude_md_path(env_dir), content).context("failed to write CLAUDE.md")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_claude_md_returns_empty_string_when_missing() {
        let dir = tempfile::tempdir().unwrap();

        let content = read_claude_md(dir.path()).unwrap();

        assert_eq!(content, "");
    }

    #[test]
    fn write_then_read_round_trips_content() {
        let dir = tempfile::tempdir().unwrap();

        write_claude_md(dir.path(), "# Instructions\n\nDo the thing.\n").unwrap();
        let content = read_claude_md(dir.path()).unwrap();

        assert_eq!(content, "# Instructions\n\nDo the thing.\n");
    }

    #[test]
    fn write_claude_md_overwrites_prior_content() {
        let dir = tempfile::tempdir().unwrap();

        write_claude_md(dir.path(), "old content").unwrap();
        write_claude_md(dir.path(), "new content").unwrap();
        let content = read_claude_md(dir.path()).unwrap();

        assert_eq!(content, "new content");
    }
}
