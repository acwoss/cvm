use cvm_core::env;
use cvm_core::ui::{
    self, EnvVarSource, EnvironmentDetail, EnvironmentSummary, HookSummary, SkillContent,
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
