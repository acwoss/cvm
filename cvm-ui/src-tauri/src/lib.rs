mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_environments,
            commands::get_environment_detail,
            commands::check_auth_status,
            commands::reveal_env_var,
            commands::open_in_claude,
            commands::create_environment,
            commands::remove_environment,
            commands::write_config_section,
            commands::write_env_var,
            commands::remove_env_var,
            commands::add_marketplace,
            commands::remove_marketplace,
            commands::install_plugin,
            commands::uninstall_plugin,
            commands::enable_plugin,
            commands::disable_plugin,
            commands::get_skill_content,
            commands::write_skill_content,
            commands::create_skill,
            commands::delete_skill,
            commands::get_agent_content,
            commands::write_agent_content,
            commands::create_agent,
            commands::delete_agent,
            commands::list_hooks,
            commands::get_hook,
            commands::write_hook,
            commands::delete_hook,
            commands::set_hook_enabled,
            commands::check_ui_update,
            commands::apply_ui_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
