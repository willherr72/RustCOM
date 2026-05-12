use std::fs;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::port::worker::TabId;
use crate::script::engine::ScriptEngine;
use crate::storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub id: String,   // file name without extension, e.g. "heartbeat"
    pub source: String,
}

fn scripts_dir() -> AppResult<std::path::PathBuf> {
    let dir = storage::data_dir()?.join("scripts");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| AppError::Io(e.to_string()))?;
    }
    Ok(dir)
}

fn validate_id(id: &str) -> AppResult<()> {
    if id.is_empty() || id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(AppError::Invalid(format!("invalid script id: {id}")));
    }
    Ok(())
}

#[tauri::command]
pub fn list_scripts() -> AppResult<Vec<Script>> {
    let dir = scripts_dir()?;
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| AppError::Io(e.to_string()))? {
        let entry = entry.map_err(|e| AppError::Io(e.to_string()))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("lua") {
            continue;
        }
        let id = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        let source = fs::read_to_string(&path).map_err(|e| AppError::Io(e.to_string()))?;
        out.push(Script { id, source });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

#[tauri::command]
pub fn save_script(item: Script) -> AppResult<Vec<Script>> {
    validate_id(&item.id)?;
    let path = scripts_dir()?.join(format!("{}.lua", item.id));
    fs::write(&path, item.source).map_err(|e| AppError::Io(e.to_string()))?;
    list_scripts()
}

#[tauri::command]
pub fn delete_script(id: String) -> AppResult<Vec<Script>> {
    validate_id(&id)?;
    let path = scripts_dir()?.join(format!("{}.lua", id));
    if path.exists() {
        fs::remove_file(&path).map_err(|e| AppError::Io(e.to_string()))?;
    }
    list_scripts()
}

#[tauri::command]
pub fn script_run(tab_id: TabId, source: String, engine: State<'_, ScriptEngine>) -> AppResult<()> {
    engine.run(tab_id, source)
}

#[tauri::command]
pub fn script_stop(engine: State<'_, ScriptEngine>) -> AppResult<()> {
    engine.stop()
}
