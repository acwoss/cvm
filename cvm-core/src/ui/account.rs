use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AccountInfo {
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub organization_name: Option<String>,
    pub seat_tier: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeJson {
    #[serde(default, rename = "oauthAccount")]
    oauth_account: Option<OauthAccount>,
}

#[derive(Debug, Deserialize)]
struct OauthAccount {
    #[serde(default, rename = "emailAddress")]
    email_address: Option<String>,
    #[serde(default, rename = "displayName")]
    display_name: Option<String>,
    #[serde(default, rename = "organizationName")]
    organization_name: Option<String>,
    #[serde(default, rename = "seatTier")]
    seat_tier: Option<String>,
}

/// Lê a conta autenticada de `.claude.json`'s `oauthAccount` - nunca toca
/// em `.credentials.json` (tokens). Retorna `None` para um ambiente
/// anônimo (sem `.claude.json`, ou sem `oauthAccount`), não um erro.
pub fn read_account(env_dir: &Path) -> Result<Option<AccountInfo>> {
    let path = env_dir.join(".claude.json");
    if !path.is_file() {
        return Ok(None);
    }
    let parsed: ClaudeJson = serde_json::from_str(&fs::read_to_string(&path)?)?;
    Ok(parsed.oauth_account.map(|a| AccountInfo {
        email: a.email_address,
        display_name: a.display_name,
        organization_name: a.organization_name,
        seat_tier: a.seat_tier,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_account_info_from_oauth_account_block() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(".claude.json"),
            r#"{"oauthAccount":{"emailAddress":"dev@example.com","displayName":"Dev","organizationName":"Acme","seatTier":"team_standard"}}"#,
        )
        .unwrap();

        let account = read_account(dir.path()).unwrap().unwrap();

        assert_eq!(account.email.as_deref(), Some("dev@example.com"));
        assert_eq!(account.display_name.as_deref(), Some("Dev"));
        assert_eq!(account.organization_name.as_deref(), Some("Acme"));
        assert_eq!(account.seat_tier.as_deref(), Some("team_standard"));
    }

    #[test]
    fn returns_none_for_anonymous_environment() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_account(dir.path()).unwrap(), None);
    }

    #[test]
    fn returns_none_when_claude_json_has_no_oauth_account() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".claude.json"), r#"{"numStartups":3}"#).unwrap();
        assert_eq!(read_account(dir.path()).unwrap(), None);
    }
}
