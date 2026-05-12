use serde::Serialize;
use tauri::{AppHandle, State};

use crate::error::AppResult;
use crate::hex;
use crate::port::config::{LineEnding, PortConfig};
use crate::port::manager::PortManager;
use crate::port::worker::TabId;

#[derive(Serialize)]
pub struct PortInfo {
    pub name: String,
    pub kind: String,
}

#[tauri::command]
pub fn list_ports() -> AppResult<Vec<PortInfo>> {
    let raw = PortManager::list_ports()?;
    Ok(raw
        .into_iter()
        .map(|p| {
            let kind = match p.port_type {
                serialport::SerialPortType::UsbPort(info) => {
                    format!("USB {:04x}:{:04x}", info.vid, info.pid)
                }
                serialport::SerialPortType::PciPort => "PCI".to_string(),
                serialport::SerialPortType::BluetoothPort => "Bluetooth".to_string(),
                serialport::SerialPortType::Unknown => "".to_string(),
            };
            PortInfo { name: p.port_name, kind }
        })
        .collect())
}

#[tauri::command]
pub fn open_port(
    tab_id: TabId,
    config: PortConfig,
    manager: State<'_, PortManager>,
    app: AppHandle,
) -> AppResult<()> {
    manager.open(tab_id, config, app)
}

#[tauri::command]
pub fn close_port(tab_id: TabId, manager: State<'_, PortManager>) -> AppResult<()> {
    manager.close(tab_id)
}

#[tauri::command]
pub fn send_text(
    tab_id: TabId,
    text: String,
    line_ending: LineEnding,
    manager: State<'_, PortManager>,
) -> AppResult<()> {
    let mut bytes = text.into_bytes();
    bytes.extend_from_slice(line_ending.as_bytes());
    manager.send(tab_id, bytes)
}

#[tauri::command]
pub fn send_hex(
    tab_id: TabId,
    input: String,
    manager: State<'_, PortManager>,
) -> AppResult<()> {
    let bytes = hex::parse_hex_input(&input)
        .map_err(crate::error::AppError::Invalid)?;
    manager.send(tab_id, bytes)
}

#[tauri::command]
pub fn set_dtr(
    tab_id: TabId,
    state: bool,
    manager: State<'_, PortManager>,
) -> AppResult<()> {
    manager.set_dtr(tab_id, state)
}

#[tauri::command]
pub fn set_rts(
    tab_id: TabId,
    state: bool,
    manager: State<'_, PortManager>,
) -> AppResult<()> {
    manager.set_rts(tab_id, state)
}
