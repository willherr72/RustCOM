use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::port::config::LineEnding;
use crate::storage;

const FILE: &str = "macros.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MacroMode { Ascii, Hex }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Macro {
    pub id: String,
    pub name: String,
    pub mode: MacroMode,
    pub payload: String,
    #[serde(default)]
    pub line_ending: Option<LineEnding>,
    #[serde(default)]
    pub hotkey: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct MacroFile {
    #[serde(default)]
    macros: Vec<Macro>,
}

#[tauri::command]
pub fn list_macros() -> AppResult<Vec<Macro>> {
    let file: MacroFile = storage::read_json(FILE)?;
    Ok(file.macros)
}

#[tauri::command]
pub fn save_macro(item: Macro) -> AppResult<Vec<Macro>> {
    let mut file: MacroFile = storage::read_json(FILE)?;
    if item.id.is_empty() {
        return Err(AppError::Invalid("macro id is empty".to_string()));
    }
    if let Some(slot) = file.macros.iter_mut().find(|m| m.id == item.id) {
        *slot = item;
    } else {
        file.macros.push(item);
    }
    storage::write_json(FILE, &file)?;
    Ok(file.macros)
}

#[tauri::command]
pub fn delete_macro(id: String) -> AppResult<Vec<Macro>> {
    let mut file: MacroFile = storage::read_json(FILE)?;
    file.macros.retain(|m| m.id != id);
    storage::write_json(FILE, &file)?;
    Ok(file.macros)
}
