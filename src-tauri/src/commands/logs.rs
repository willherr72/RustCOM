use std::fs;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Direction { Rx, Tx }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub ts_ms: i64,
    pub direction: Direction,
    pub bytes: Vec<u8>,
}

#[tauri::command]
pub fn save_log(path: String, entries: Vec<LogEntry>) -> AppResult<String> {
    if path.trim().is_empty() {
        return Err(AppError::Invalid("path is empty".to_string()));
    }
    let mut body = String::new();
    for e in &entries {
        let dir = match e.direction {
            Direction::Rx => "RX",
            Direction::Tx => "TX",
        };
        let text = String::from_utf8_lossy(&e.bytes);
        body.push_str(&format!("[{}] {}: {}\n", e.ts_ms, dir, text));
    }
    fs::write(&path, body).map_err(|e| AppError::Io(e.to_string()))?;
    Ok(format!("wrote {} entries to {}", entries.len(), path))
}
