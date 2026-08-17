use cvm_core::env;
use cvm_core::ui::{self, EnvVarSource, EnvironmentDetail, EnvironmentSummary};

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
