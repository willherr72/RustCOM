pub mod commands;
pub mod error;
pub mod hex;
pub mod port;
pub mod storage;

use port::manager::PortManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(PortManager::new())
        .invoke_handler(tauri::generate_handler![
            commands::ports::list_ports,
            commands::ports::open_port,
            commands::ports::close_port,
            commands::ports::send_text,
            commands::ports::send_hex,
            commands::ports::set_dtr,
            commands::ports::set_rts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
