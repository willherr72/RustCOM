use serialport::{SerialPort, SerialPortInfo};
use std::sync::{Arc, Mutex};
use regex::Regex;
use chrono::Local;

use crate::hex;
use crate::logging::{self, DataLogEntry};
use crate::serial::*;

pub const MAX_BUFFER_SIZE: usize = 100_000;
pub const BUFFER_DRAIN_SIZE: usize = 10_000;
pub const SERIAL_READ_BUFFER_SIZE: usize = 1024;
pub const DEFAULT_REPAINT_INTERVAL_MS: u64 = 50;
pub const DEFAULT_PORT_SCAN_INTERVAL_MS: u64 = 3000;
pub const DEFAULT_RECONNECT_DELAY_MS: u64 = 2000;
pub const SIDEBAR_WIDTH: f32 = 240.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Ascii,
    Hex,
    Both,
}

pub struct ComAnalyzerApp {
    // Connection settings
    pub available_ports: Vec<SerialPortInfo>,
    pub selected_port: Option<String>,
    pub baud_rate: String,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
    pub flow_control: FlowControl,
    pub connected: bool,
    pub serial_port: Option<Arc<Mutex<Box<dyn SerialPort>>>>,

    // Display
    pub receive_buffer: Vec<u8>,
    pub receive_buffer_display: String,
    pub send_buffer: String,
    pub view_mode: ViewMode,
    pub strip_ansi: bool,

    // Advanced features
    pub auto_scroll: bool,
    pub dtr_state: bool,
    pub rts_state: bool,
    pub auto_reconnect: bool,
    pub reconnect_delay_ms: u64,
    pub reconnecting: bool,
    pub last_reconnect_attempt: std::time::Instant,

    // Port scanning
    pub auto_scan_ports: bool,
    pub port_scan_interval_ms: u64,
    pub last_port_scan: std::time::Instant,

    // Logging
    pub logging_enabled: bool,
    pub log_file_path: String,
    pub log_entries: Vec<DataLogEntry>,

    // Filtering
    pub filter_enabled: bool,
    pub filter_pattern: String,
    pub filter_regex: Option<Regex>,

    // Virtual COM
    pub virtual_com_port: Option<String>,

    // Send options
    pub send_mode: SendMode,
    pub line_ending: LineEnding,

    // UI state
    pub error_message: Option<String>,
    pub bytes_received: usize,
    pub bytes_sent: usize,
}

impl Default for ComAnalyzerApp {
    fn default() -> Self {
        Self {
            available_ports: serialport::available_ports().unwrap_or_default(),
            selected_port: None,
            baud_rate: "9600".to_string(),
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            parity: Parity::None,
            flow_control: FlowControl::None,
            connected: false,
            serial_port: None,
            receive_buffer: Vec::new(),
            receive_buffer_display: String::new(),
            send_buffer: String::new(),
            view_mode: ViewMode::Ascii,
            strip_ansi: true,
            auto_scroll: true,
            dtr_state: false,
            rts_state: false,
            auto_reconnect: false,
            reconnect_delay_ms: DEFAULT_RECONNECT_DELAY_MS,
            reconnecting: false,
            last_reconnect_attempt: std::time::Instant::now(),
            auto_scan_ports: true,
            port_scan_interval_ms: DEFAULT_PORT_SCAN_INTERVAL_MS,
            last_port_scan: std::time::Instant::now(),
            logging_enabled: false,
            log_file_path: format!("rustcom_{}.log", Local::now().format("%Y%m%d_%H%M%S")),
            log_entries: Vec::new(),
            filter_enabled: false,
            filter_pattern: String::new(),
            filter_regex: None,
            virtual_com_port: None,
            send_mode: SendMode::Ascii,
            line_ending: LineEnding::CrLf,
            error_message: None,
            bytes_received: 0,
            bytes_sent: 0,
        }
    }
}

impl ComAnalyzerApp {
    pub fn update_display_buffer(&mut self) {
        self.receive_buffer_display = match self.view_mode {
            ViewMode::Ascii => {
                let raw = String::from_utf8_lossy(&self.receive_buffer).to_string();
                if self.strip_ansi {
                    hex::strip_ansi_codes(&raw)
                } else {
                    raw
                }
            }
            ViewMode::Hex => hex::format_hex(&self.receive_buffer),
            ViewMode::Both => {
                let ascii_raw = String::from_utf8_lossy(&self.receive_buffer).to_string();
                let ascii = if self.strip_ansi {
                    hex::strip_ansi_codes(&ascii_raw)
                } else {
                    ascii_raw
                };
                let hex_view = hex::format_hex(&self.receive_buffer);
                format!("=== HEX ===\n{}\n\n=== ASCII ===\n{}", hex_view, ascii)
            }
        };
    }

    pub fn matches_filter(&self, data: &[u8]) -> bool {
        if let Some(regex) = &self.filter_regex {
            let text = String::from_utf8_lossy(data);
            regex.is_match(&text)
        } else {
            true
        }
    }

    pub fn update_filter(&mut self) {
        if self.filter_pattern.is_empty() {
            self.filter_regex = None;
        } else {
            match Regex::new(&self.filter_pattern) {
                Ok(regex) => {
                    self.filter_regex = Some(regex);
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Invalid regex: {}", e));
                }
            }
        }
    }

    pub fn save_log(&mut self) {
        match logging::save_log(&self.log_entries, &self.log_file_path) {
            Ok(msg) => self.error_message = Some(msg),
            Err(msg) => self.error_message = Some(msg),
        }
    }

    pub fn save_buffer(&self) {
        logging::save_buffer(&self.receive_buffer_display);
    }

    pub fn create_virtual_com(&mut self) {
        match crate::virtual_com::create_loopback_pair() {
            Ok((port1, port2)) => {
                self.virtual_com_port = Some(format!("{} <-> {}", port1, port2));
                self.error_message =
                    Some(format!("Created virtual COM pair: {} and {}", port1, port2));
            }
            Err(e) => {
                self.error_message = Some(e);
            }
        }
    }
}
