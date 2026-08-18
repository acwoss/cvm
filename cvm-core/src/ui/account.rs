use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthMethod {
    Oauth,
    ApiKey,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub auth_method: AuthMethod,
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

const API_KEY_ENV_VAR: &str = "ANTHROPIC_API_KEY";

/// Lê a conta autenticada: prioriza `oauthAccount` de `.claude.json`. Na
/// ausência dele, verifica se `ANTHROPIC_API_KEY` está configurada (`.env`
/// ou bloco `env` do settings.json) via `list_env_var_summaries` - nunca lê
/// o *valor* da chave, só sua presença. Retorna `None` apenas quando nenhum
/// dos dois existe (ambiente de fato anônimo). Nunca toca `.credentials.json`
/// (tokens).
pub fn read_account(env_dir: &Path) -> Result<Option<AccountInfo>> {
    let path = env_dir.join(".claude.json");
    if path.is_file() {
        let parsed: ClaudeJson = serde_json::from_str(&fs::read_to_string(&path)?)?;
        if let Some(a) = parsed.oauth_account {
            return Ok(Some(AccountInfo {
                auth_method: AuthMethod::Oauth,
                email: a.email_address,
                display_name: a.display_name,
                organization_name: a.organization_name,
                seat_tier: a.seat_tier,
            }));
        }
    }

    let has_api_key = config::list_env_var_summaries(env_dir)?
        .iter()
        .any(|v| v.key == API_KEY_ENV_VAR);
    if has_api_key {
        return Ok(Some(AccountInfo {
            auth_method: AuthMethod::ApiKey,
            email: None,
            display_name: None,
            organization_name: None,
            seat_tier: None,
        }));
    }

    Ok(None)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub logged_in: bool,
    pub auth_method: String,
    #[serde(default)]
    pub api_provider: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub org_id: Option<String>,
    #[serde(default)]
    pub org_name: Option<String>,
    #[serde(default)]
    pub subscription_type: Option<String>,
    #[serde(default)]
    pub api_key_source: Option<String>,
}

/// Faz o parsing puro do stdout de `claude auth status` (JSON). Separado de
/// `check_auth_status` (em `ui/mod.rs`, que dispara o subprocesso) para ser
/// testável sem depender do binário `claude` instalado.
pub fn parse_auth_status(stdout: &[u8]) -> Result<AuthStatus> {
    let text = String::from_utf8_lossy(stdout);
    serde_json::from_str(text.trim())
        .with_context(|| format!("failed to parse 'claude auth status' output: {text}"))
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

        assert_eq!(account.auth_method, AuthMethod::Oauth);
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
    fn returns_none_when_claude_json_has_no_oauth_account_and_no_api_key() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".claude.json"), r#"{"numStartups":3}"#).unwrap();
        assert_eq!(read_account(dir.path()).unwrap(), None);
    }

    #[test]
    fn reports_api_key_auth_method_when_anthropic_api_key_is_in_dotenv() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(".env"),
            "ANTHROPIC_API_KEY=sk-ant-example\n",
        )
        .unwrap();

        let account = read_account(dir.path()).unwrap().unwrap();

        assert_eq!(account.auth_method, AuthMethod::ApiKey);
        assert_eq!(account.email, None);
    }

    #[test]
    fn reports_api_key_auth_method_when_anthropic_api_key_is_in_settings_json_env_block() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("settings.json"),
            r#"{"env":{"ANTHROPIC_API_KEY":"sk-ant-example"}}"#,
        )
        .unwrap();

        let account = read_account(dir.path()).unwrap().unwrap();

        assert_eq!(account.auth_method, AuthMethod::ApiKey);
    }

    #[test]
    fn oauth_account_takes_precedence_over_api_key_env_var() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(".claude.json"),
            r#"{"oauthAccount":{"emailAddress":"dev@example.com"}}"#,
        )
        .unwrap();
        fs::write(
            dir.path().join(".env"),
            "ANTHROPIC_API_KEY=sk-ant-example\n",
        )
        .unwrap();

        let account = read_account(dir.path()).unwrap().unwrap();

        assert_eq!(account.auth_method, AuthMethod::Oauth);
        assert_eq!(account.email.as_deref(), Some("dev@example.com"));
    }

    #[test]
    fn parse_auth_status_reads_oauth_logged_in_status() {
        let json = br#"{"loggedIn":true,"authMethod":"claude.ai","apiProvider":"firstParty","email":"dev@example.com","orgId":"org_1","orgName":"Acme","subscriptionType":"team"}"#;

        let status = parse_auth_status(json).unwrap();

        assert!(status.logged_in);
        assert_eq!(status.auth_method, "claude.ai");
        assert_eq!(status.email.as_deref(), Some("dev@example.com"));
        assert_eq!(status.org_name.as_deref(), Some("Acme"));
        assert_eq!(status.subscription_type.as_deref(), Some("team"));
        assert_eq!(status.api_key_source, None);
    }

    #[test]
    fn parse_auth_status_reads_logged_out_status() {
        let json = br#"{"loggedIn":false,"authMethod":"none","apiProvider":"firstParty"}"#;

        let status = parse_auth_status(json).unwrap();

        assert!(!status.logged_in);
        assert_eq!(status.email, None);
    }

    #[test]
    fn parse_auth_status_reads_api_key_status() {
        let json = br#"{"loggedIn":true,"authMethod":"api_key","apiProvider":"firstParty","apiKeySource":"ANTHROPIC_API_KEY"}"#;

        let status = parse_auth_status(json).unwrap();

        assert!(status.logged_in);
        assert_eq!(status.auth_method, "api_key");
        assert_eq!(status.api_key_source.as_deref(), Some("ANTHROPIC_API_KEY"));
    }

    #[test]
    fn parse_auth_status_errors_on_invalid_json() {
        assert!(parse_auth_status(b"not json").is_err());
    }
}
