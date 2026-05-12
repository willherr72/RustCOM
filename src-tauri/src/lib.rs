pub mod commands;
pub mod error;
pub mod hex;
pub mod port;
pub mod script;
pub mod storage;

use tauri::Manager;

use port::manager::PortManager;
use script::engine::ScriptEngine;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    log::info!("RustCOM starting");
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(PortManager::new())
        .manage(ScriptEngine::new())
        .setup(|app| {
            let handle = app.handle().clone();
            let engine = handle.state::<ScriptEngine>();
            engine.start(handle.clone()).ok();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ports::list_ports,
            commands::ports::open_port,
            commands::ports::close_port,
            commands::ports::send_text,
            commands::ports::send_hex,
            commands::ports::set_dtr,
            commands::ports::set_rts,
            commands::macros::list_macros,
            commands::macros::save_macro,
            commands::macros::delete_macro,
            commands::logs::save_log,
            commands::settings::load_settings,
            commands::settings::save_settings,
            commands::scripts::list_scripts,
            commands::scripts::save_script,
            commands::scripts::delete_script,
            commands::scripts::script_run,
            commands::scripts::script_stop,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
