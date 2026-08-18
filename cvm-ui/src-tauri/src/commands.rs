use anyhow::Context;
use cvm_core::env;
use cvm_core::ui::{
    self, AuthStatus, EnvVarSource, EnvironmentDetail, EnvironmentSummary, HookSummary,
    SkillContent,
};

/// Erro serializável para o frontend - `anyhow::Error` não implementa
/// `serde::Serialize`, então toda falha de `cvm-core` é convertida para sua
/// mensagem formatada (`{:#}`, com a cadeia de causas) antes de atravessar
/// a ponte do Tauri.
#[derive(Debug, serde::Serialize)]
pub struct CommandError {
    message: String,
}

impl From<anyhow::Error> for CommandError {
    fn from(err: anyhow::Error) -> Self {
        CommandError {
            message: format!("{err:#}"),
        }
    }
}

#[tauri::command]
pub fn list_environments() -> Result<Vec<EnvironmentSummary>, CommandError> {
    Ok(ui::list_environment_summaries()?)
}

#[tauri::command]
pub fn get_environment_detail(name: String) -> Result<EnvironmentDetail, CommandError> {
    Ok(ui::environment_detail(&name)?)
}

#[tauri::command]
pub fn check_auth_status(name: String) -> Result<AuthStatus, CommandError> {
    Ok(ui::check_auth_status(&name)?)
}

#[tauri::command]
pub fn reveal_env_var(
    name: String,
    source: EnvVarSource,
    key: String,
) -> Result<String, CommandError> {
    Ok(ui::reveal_env_var(&name, source, &key)?)
}

#[tauri::command]
pub fn open_in_claude(name: String) -> Result<(), CommandError> {
    Ok(env::open_env_detached(&name)?)
}

#[tauri::command]
pub fn create_environment(
    name: String,
    anonymous: bool,
    inherit: bool,
    open: bool,
) -> Result<(), CommandError> {
    env::create_env(&name, anonymous, inherit)?;
    if open {
        env::open_env_detached(&name)?;
    }
    Ok(())
}

#[tauri::command]
pub fn remove_environment(name: String) -> Result<(), CommandError> {
    env::remove_env(&name)?;
    Ok(())
}

#[tauri::command]
pub fn write_config_section(
    name: String,
    allowed_tools: Vec<String>,
    denied_tools: Vec<String>,
) -> Result<(), CommandError> {
    let dir = env::ensure_env_exists(&name)?;
    ui::write_config_section(&dir, &allowed_tools, &denied_tools)?;
    Ok(())
}

#[tauri::command]
pub fn write_env_var(
    name: String,
    source: EnvVarSource,
    key: String,
    value: String,
) -> Result<(), CommandError> {
    let dir = env::ensure_env_exists(&name)?;
    ui::write_env_var(&dir, source, &key, &value)?;
    Ok(())
}

#[tauri::command]
pub fn remove_env_var(name: String, source: EnvVarSource, key: String) -> Result<(), CommandError> {
    let dir = env::ensure_env_exists(&name)?;
    ui::remove_env_var(&dir, source, &key)?;
    Ok(())
}

fn run_claude_plugin_command(name: &str, args: &[&str]) -> Result<String, CommandError> {
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let output = env::run_claude_command(name, &args)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("{}", stderr.trim()).into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[tauri::command]
pub fn add_marketplace(name: String, source: String) -> Result<String, CommandError> {
    run_claude_plugin_command(&name, &["plugin", "marketplace", "add", &source])
}

#[tauri::command]
pub fn remove_marketplace(name: String, marketplace: String) -> Result<String, CommandError> {
    run_claude_plugin_command(&name, &["plugin", "marketplace", "remove", &marketplace])
}

#[tauri::command]
pub fn install_plugin(name: String, plugin: String) -> Result<String, CommandError> {
    run_claude_plugin_command(&name, &["plugin", "install", &plugin, "-y"])
}

#[tauri::command]
pub fn uninstall_plugin(name: String, plugin: String) -> Result<String, CommandError> {
    run_claude_plugin_command(&name, &["plugin", "uninstall", &plugin])
}

#[tauri::command]
pub fn enable_plugin(name: String, plugin: String) -> Result<String, CommandError> {
    run_claude_plugin_command(&name, &["plugin", "enable", &plugin])
}

#[tauri::command]
pub fn disable_plugin(name: String, plugin: String) -> Result<String, CommandError> {
    run_claude_plugin_command(&name, &["plugin", "disable", &plugin])
}

#[tauri::command]
pub fn get_skill_content(env_name: String, id: String) -> Result<SkillContent, CommandError> {
    let dir = env::ensure_env_exists(&env_name)?;
    Ok(ui::read_skill_content(&dir, &id)?)
}

#[tauri::command]
pub fn write_skill_content(
    env_name: String,
    id: String,
    content: SkillContent,
) -> Result<(), CommandError> {
    let dir = env::ensure_env_exists(&env_name)?;
    ui::write_skill_content(&dir, &id, &content)?;
    Ok(())
}

#[tauri::command]
pub fn create_skill(
    env_name: String,
    id: String,
    name: String,
    description: String,
) -> Result<(), CommandError> {
    let dir = env::ensure_env_exists(&env_name)?;
    ui::create_skill(&dir, &id, &name, &description)?;
    Ok(())
}

#[tauri::command]
pub fn delete_skill(env_name: String, id: String) -> Result<(), CommandError> {
    let dir = env::ensure_env_exists(&env_name)?;
    ui::delete_skill(&dir, &id)?;
    Ok(())
}

#[tauri::command]
pub fn get_agent_content(env_name: String, id: String) -> Result<SkillContent, CommandError> {
    let dir = env::ensure_env_exists(&env_name)?;
    Ok(ui::read_agent_content(&dir, &id)?)
}

#[tauri::command]
pub fn write_agent_content(
    env_name: String,
    id: String,
    content: SkillContent,
) -> Result<(), CommandError> {
    let dir = env::ensure_env_exists(&env_name)?;
    ui::write_agent_content(&dir, &id, &content)?;
    Ok(())
}

#[tauri::command]
pub fn create_agent(
    env_name: String,
    id: String,
    name: String,
    description: String,
) -> Result<(), CommandError> {
    let dir = env::ensure_env_exists(&env_name)?;
    ui::create_agent(&dir, &id, &name, &description)?;
    Ok(())
}

#[tauri::command]
pub fn delete_agent(env_name: String, id: String) -> Result<(), CommandError> {
    let dir = env::ensure_env_exists(&env_name)?;
    ui::delete_agent(&dir, &id)?;
    Ok(())
}

#[tauri::command]
pub fn list_hooks() -> Result<Vec<HookSummary>, CommandError> {
    Ok(ui::list_hooks()?)
}

#[tauri::command]
pub fn get_hook(event: String) -> Result<Option<String>, CommandError> {
    Ok(ui::read_hook(&event)?)
}

#[tauri::command]
pub fn write_hook(event: String, content: String) -> Result<(), CommandError> {
    ui::write_hook(&event, &content)?;
    Ok(())
}

#[tauri::command]
pub fn delete_hook(event: String) -> Result<(), CommandError> {
    ui::delete_hook(&event)?;
    Ok(())
}

#[tauri::command]
pub fn set_hook_enabled(event: String, enabled: bool) -> Result<(), CommandError> {
    ui::set_hook_enabled(&event, enabled)?;
    Ok(())
}

#[derive(Debug, serde::Serialize)]
pub struct UpdateInfo {
    pub current: String,
    pub latest: String,
}

const REPO: &str = "acwoss/cvm";
const UI_BIN_NAME: &str = if cfg!(windows) {
    "cvm-ui.exe"
} else {
    "cvm-ui"
};

#[tauri::command]
pub fn check_ui_update() -> Result<Option<UpdateInfo>, CommandError> {
    let current = env!("CARGO_PKG_VERSION");
    let latest_tag = cvm_core::update::fetch_latest_tag(REPO, 10)?;
    let latest = latest_tag.trim_start_matches('v').to_string();
    if latest == current {
        Ok(None)
    } else {
        Ok(Some(UpdateInfo {
            current: current.to_string(),
            latest,
        }))
    }
}

#[tauri::command]
pub fn apply_ui_update() -> Result<(), CommandError> {
    let latest_tag = cvm_core::update::fetch_latest_tag(REPO, 10)?;
    let target = cvm_core::update::target_triple_for_ui()?;
    let asset = cvm_core::update::asset_name("cvm-ui", target);
    let url = format!("https://github.com/{REPO}/releases/download/{latest_tag}/{asset}");

    let tmp_dir = std::env::temp_dir().join(format!("cvm-ui-self-update-{}", std::process::id()));
    std::fs::create_dir_all(&tmp_dir).context("failed to create temp dir")?;
    let cleanup = |dir: &std::path::Path| {
        let _ = std::fs::remove_dir_all(dir);
    };

    let archive_path = tmp_dir.join(&asset);
    if let Err(err) = cvm_core::update::download_asset(&url, &archive_path) {
        cleanup(&tmp_dir);
        return Err(err.into());
    }
    if let Err(err) = cvm_core::update::extract_asset(&archive_path, &tmp_dir) {
        cleanup(&tmp_dir);
        return Err(err.into());
    }
    let new_binary = match cvm_core::update::find_binary(&tmp_dir, UI_BIN_NAME) {
        Ok(path) => path,
        Err(err) => {
            cleanup(&tmp_dir);
            return Err(err.into());
        }
    };

    let result = cvm_core::update::replace_running_binary(&new_binary);
    cleanup(&tmp_dir);
    result?;
    Ok(())
}
