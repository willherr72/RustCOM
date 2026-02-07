use std::io::Write as IoWrite;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::Local;

use crate::app::ComAnalyzerApp;
use crate::logging::{self, Direction};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataBits {
    Five,
    Six,
    Seven,
    Eight,
}

impl DataBits {
    pub fn to_serial(self) -> serialport::DataBits {
        match self {
            DataBits::Five => serialport::DataBits::Five,
            DataBits::Six => serialport::DataBits::Six,
            DataBits::Seven => serialport::DataBits::Seven,
            DataBits::Eight => serialport::DataBits::Eight,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            DataBits::Five => "5",
            DataBits::Six => "6",
            DataBits::Seven => "7",
            DataBits::Eight => "8",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StopBits {
    One,
    Two,
}

impl StopBits {
    pub fn to_serial(self) -> serialport::StopBits {
        match self {
            StopBits::One => serialport::StopBits::One,
            StopBits::Two => serialport::StopBits::Two,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            StopBits::One => "1",
            StopBits::Two => "2",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Parity {
    None,
    Even,
    Odd,
}

impl Parity {
    pub fn to_serial(self) -> serialport::Parity {
        match self {
            Parity::None => serialport::Parity::None,
            Parity::Even => serialport::Parity::Even,
            Parity::Odd => serialport::Parity::Odd,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Parity::None => "None",
            Parity::Even => "Even",
            Parity::Odd => "Odd",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlowControl {
    None,
    Software,
    Hardware,
}

impl FlowControl {
    pub fn to_serial(self) -> serialport::FlowControl {
        match self {
            FlowControl::None => serialport::FlowControl::None,
            FlowControl::Software => serialport::FlowControl::Software,
            FlowControl::Hardware => serialport::FlowControl::Hardware,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            FlowControl::None => "None",
            FlowControl::Software => "Software",
            FlowControl::Hardware => "Hardware",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineEnding {
    None,
    Cr,
    Lf,
    CrLf,
}

impl LineEnding {
    pub fn as_str(&self) -> &str {
        match self {
            LineEnding::None => "None",
            LineEnding::Cr => "\\r",
            LineEnding::Lf => "\\n",
            LineEnding::CrLf => "\\r\\n",
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            LineEnding::None => b"",
            LineEnding::Cr => b"\r",
            LineEnding::Lf => b"\n",
            LineEnding::CrLf => b"\r\n",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SendMode {
    Ascii,
    Hex,
}

impl ComAnalyzerApp {
    pub fn connect(&mut self) {
        self.error_message = None;

        let Some(port_name) = &self.selected_port else {
            self.error_message = Some("Please select a port".to_string());
            return;
        };

        let baud_rate: u32 = match self.baud_rate.parse() {
            Ok(rate) => rate,
            Err(_) => {
                self.error_message = Some("Invalid baud rate".to_string());
                return;
            }
        };

        match serialport::new(port_name, baud_rate)
            .data_bits(self.data_bits.to_serial())
            .stop_bits(self.stop_bits.to_serial())
            .parity(self.parity.to_serial())
            .flow_control(self.flow_control.to_serial())
            .timeout(Duration::from_millis(10))
            .open()
        {
            Ok(port) => {
                self.serial_port = Some(Arc::new(Mutex::new(port)));
                self.connected = true;
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                let msg = format!(
                    "[{}] Connected to {} at {} baud\n",
                    timestamp, port_name, baud_rate
                );
                self.receive_buffer.extend_from_slice(msg.as_bytes());
                self.update_display_buffer();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to open port: {}", e));
            }
        }
    }

    pub fn disconnect(&mut self) {
        self.serial_port = None;
        self.connected = false;
        self.reconnecting = false;
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let msg = format!("[{}] Disconnected\n", timestamp);
        self.receive_buffer.extend_from_slice(msg.as_bytes());
        self.update_display_buffer();
    }

    pub fn send_data(&mut self) {
        let mut data = self.send_buffer.clone().into_bytes();
        data.extend_from_slice(self.line_ending.as_bytes());

        let send_result = if let Some(port) = &self.serial_port {
            if let Ok(mut port_guard) = port.lock() {
                port_guard.write(&data).ok()
            } else {
                None
            }
        } else {
            None
        };

        if let Some(bytes) = send_result {
            self.bytes_sent += bytes;

            if self.logging_enabled {
                self.log_entries
                    .push(logging::create_log_entry(Direction::Sent, &data));
            }

            let msg = format!("TX: {}\n", self.send_buffer);
            self.receive_buffer.extend_from_slice(msg.as_bytes());
            self.update_display_buffer();
            self.send_buffer.clear();
        } else {
            self.error_message = Some("Send failed".to_string());
        }
    }

    pub fn send_hex_input(&mut self) {
        match crate::hex::parse_hex_input(&self.send_buffer) {
            Ok(bytes) => {
                let send_result = if let Some(port) = &self.serial_port {
                    if let Ok(mut port_guard) = port.lock() {
                        port_guard.write(&bytes).ok()
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(count) = send_result {
                    self.bytes_sent += count;

                    if self.logging_enabled {
                        self.log_entries
                            .push(logging::create_log_entry(Direction::Sent, &bytes));
                    }

                    let hex_str: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
                    let msg = format!("TX [hex]: {}\n", hex_str.join(" "));
                    self.receive_buffer.extend_from_slice(msg.as_bytes());
                    self.update_display_buffer();
                    self.send_buffer.clear();
                } else {
                    self.error_message = Some("Send failed".to_string());
                }
            }
            Err(e) => {
                self.error_message = Some(e);
            }
        }
    }

    pub fn set_dtr(&mut self, state: bool) {
        if let Some(port) = &self.serial_port {
            if let Ok(mut port_guard) = port.lock() {
                let _ = port_guard.write_data_terminal_ready(state);
            }
        }
    }

    pub fn set_rts(&mut self, state: bool) {
        if let Some(port) = &self.serial_port {
            if let Ok(mut port_guard) = port.lock() {
                let _ = port_guard.write_request_to_send(state);
            }
        }
    }
}
