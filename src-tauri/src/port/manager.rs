use std::collections::HashMap;

use parking_lot::Mutex;
use serialport::SerialPortInfo;
use tauri::AppHandle;

use crate::error::{AppError, AppResult};
use crate::port::config::PortConfig;
use crate::port::worker::{self, PortHandle, TabId, WorkerCommand};

pub struct PortManager {
    tabs: Mutex<HashMap<TabId, PortHandle>>,
}

impl PortManager {
    pub fn new() -> Self {
        Self { tabs: Mutex::new(HashMap::new()) }
    }

    pub fn list_ports() -> AppResult<Vec<SerialPortInfo>> {
        Ok(serialport::available_ports()?)
    }

    pub fn open(&self, tab_id: TabId, config: PortConfig, app: AppHandle) -> AppResult<()> {
        let mut guard = self.tabs.lock();
        if guard.contains_key(&tab_id) {
            return Err(AppError::Invalid(format!("tab {tab_id} already has an open port")));
        }
        let handle = worker::spawn(tab_id, config, app)?;
        guard.insert(tab_id, handle);
        Ok(())
    }

    pub fn close(&self, tab_id: TabId) -> AppResult<()> {
        let handle = {
            let mut guard = self.tabs.lock();
            guard.remove(&tab_id).ok_or(AppError::PortNotOpen(tab_id))?
        };
        let _ = handle.send_command(WorkerCommand::Shutdown);
        let _ = handle.join.join();
        Ok(())
    }

    pub fn send(&self, tab_id: TabId, bytes: Vec<u8>) -> AppResult<()> {
        let guard = self.tabs.lock();
        let handle = guard.get(&tab_id).ok_or(AppError::PortNotOpen(tab_id))?;
        handle.send_command(WorkerCommand::Write(bytes))
    }

    pub fn set_dtr(&self, tab_id: TabId, state: bool) -> AppResult<()> {
        let guard = self.tabs.lock();
        let handle = guard.get(&tab_id).ok_or(AppError::PortNotOpen(tab_id))?;
        handle.send_command(WorkerCommand::SetDtr(state))
    }

    pub fn set_rts(&self, tab_id: TabId, state: bool) -> AppResult<()> {
        let guard = self.tabs.lock();
        let handle = guard.get(&tab_id).ok_or(AppError::PortNotOpen(tab_id))?;
        handle.send_command(WorkerCommand::SetRts(state))
    }
}

impl Default for PortManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_unknown_tab_returns_port_not_open() {
        let mgr = PortManager::new();
        let result = mgr.close(99);
        match result {
            Err(AppError::PortNotOpen(99)) => {}
            other => panic!("expected PortNotOpen(99), got {other:?}"),
        }
    }

    #[test]
    fn list_ports_does_not_panic_in_ci() {
        // We can't assert specific contents — just that the API works on the host.
        let _ = PortManager::list_ports();
    }
}
