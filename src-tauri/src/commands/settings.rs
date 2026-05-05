use serde::{Deserialize, Serialize};

use crate::error::AppResult;
use crate::port::config::{DataBits, FlowControl, LineEnding, Parity, SendMode, StopBits};
use crate::storage;

const FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    pub baud_rate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
    pub flow_control: FlowControl,
    pub line_ending: LineEnding,
    pub send_mode: SendMode,
    pub auto_reconnect: bool,
    pub reconnect_delay_ms: u64,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            baud_rate: 9600,
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            parity: Parity::None,
            flow_control: FlowControl::None,
            line_ending: LineEnding::CrLf,
            send_mode: SendMode::Ascii,
            auto_reconnect: false,
            reconnect_delay_ms: 2000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub defaults: DefaultsConfig,
}

impl Default for Settings {
    fn default() -> Self {
        Self { defaults: DefaultsConfig::default() }
    }
}

#[tauri::command]
pub fn load_settings() -> AppResult<Settings> {
    storage::read_json(FILE)
}

#[tauri::command]
pub fn save_settings(settings: Settings) -> AppResult<Settings> {
    storage::write_json(FILE, &settings)?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_have_sensible_values() {
        let d = DefaultsConfig::default();
        assert_eq!(d.baud_rate, 9600);
        assert_eq!(d.data_bits, DataBits::Eight);
        assert_eq!(d.line_ending, LineEnding::CrLf);
    }

    #[test]
    fn settings_serde_roundtrip() {
        let s = Settings::default();
        let json = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.defaults.baud_rate, s.defaults.baud_rate);
    }
}
