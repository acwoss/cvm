mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_environments,
            commands::get_environment_detail,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
