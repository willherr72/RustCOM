#![windows_subsystem = "windows"]

mod virtual_com;

use eframe::egui;
use serialport::{SerialPort, SerialPortInfo};
use std::fs::OpenOptions;
use std::io::Write as IoWrite;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::Local;
use regex::Regex;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 700.0])
            .with_title("RustCOM - Professional COM Port Analyzer"),
        ..Default::default()
    };

    eframe::run_native(
        "RustCOM",
        options,
        Box::new(|cc| {
            // Set dark theme
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(ComAnalyzerApp::default()))
        }),
    )
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ViewMode {
    Ascii,
    Hex,
    Both,
}

#[derive(Debug, Clone)]
struct DataLogEntry {
    timestamp: String,
    direction: Direction,
    data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    Received,
    Sent,
}

struct ComAnalyzerApp {
    // Connection settings
    available_ports: Vec<SerialPortInfo>,
    selected_port: Option<String>,
    baud_rate: String,
    data_bits: DataBits,
    stop_bits: StopBits,
    parity: Parity,
    flow_control: FlowControl,
    connected: bool,
    serial_port: Option<Arc<Mutex<Box<dyn SerialPort>>>>,
    
    // Display
    receive_buffer: Vec<u8>,
    receive_buffer_display: String,
    send_buffer: String,
    view_mode: ViewMode,
    strip_ansi: bool,
    
    // Advanced features
    auto_scroll: bool,
    show_timestamps: bool,
    dtr_state: bool,
    rts_state: bool,
    auto_reconnect: bool,
    reconnect_delay_ms: u64,
    reconnecting: bool,
    last_reconnect_attempt: std::time::Instant,
    
    // Port scanning
    auto_scan_ports: bool,
    port_scan_interval_ms: u64,
    last_port_scan: std::time::Instant,
    
    // Logging
    logging_enabled: bool,
    log_file_path: String,
    log_entries: Vec<DataLogEntry>,
    
    // Filtering
    filter_enabled: bool,
    filter_pattern: String,
    filter_regex: Option<Regex>,
    
    // Protocol analyzer
    protocol_mode: ProtocolMode,
    
    // Virtual COM
    virtual_com_enabled: bool,
    virtual_com_port: Option<String>,
    
    // UI state
    error_message: Option<String>,
    show_advanced: bool,
    bytes_received: usize,
    bytes_sent: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum DataBits {
    Five,
    Six,
    Seven,
    Eight,
}

impl DataBits {
    fn to_serial(&self) -> serialport::DataBits {
        match self {
            DataBits::Five => serialport::DataBits::Five,
            DataBits::Six => serialport::DataBits::Six,
            DataBits::Seven => serialport::DataBits::Seven,
            DataBits::Eight => serialport::DataBits::Eight,
        }
    }

    fn as_str(&self) -> &str {
        match self {
            DataBits::Five => "5",
            DataBits::Six => "6",
            DataBits::Seven => "7",
            DataBits::Eight => "8",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum StopBits {
    One,
    Two,
}

impl StopBits {
    fn to_serial(&self) -> serialport::StopBits {
        match self {
            StopBits::One => serialport::StopBits::One,
            StopBits::Two => serialport::StopBits::Two,
        }
    }

    fn as_str(&self) -> &str {
        match self {
            StopBits::One => "1",
            StopBits::Two => "2",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Parity {
    None,
    Even,
    Odd,
}

impl Parity {
    fn to_serial(&self) -> serialport::Parity {
        match self {
            Parity::None => serialport::Parity::None,
            Parity::Even => serialport::Parity::Even,
            Parity::Odd => serialport::Parity::Odd,
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Parity::None => "None",
            Parity::Even => "Even",
            Parity::Odd => "Odd",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FlowControl {
    None,
    Software,
    Hardware,
}

impl FlowControl {
    fn to_serial(&self) -> serialport::FlowControl {
        match self {
            FlowControl::None => serialport::FlowControl::None,
            FlowControl::Software => serialport::FlowControl::Software,
            FlowControl::Hardware => serialport::FlowControl::Hardware,
        }
    }

    fn as_str(&self) -> &str {
        match self {
            FlowControl::None => "None",
            FlowControl::Software => "Software",
            FlowControl::Hardware => "Hardware",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ProtocolMode {
    None,
    ModbusRTU,
    ModbusASCII,
    Custom,
}

impl ProtocolMode {
    fn as_str(&self) -> &str {
        match self {
            ProtocolMode::None => "None",
            ProtocolMode::ModbusRTU => "Modbus RTU",
            ProtocolMode::ModbusASCII => "Modbus ASCII",
            ProtocolMode::Custom => "Custom",
        }
    }
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
            show_timestamps: false,
            dtr_state: false,
            rts_state: false,
            auto_reconnect: false,
            reconnect_delay_ms: 2000,
            reconnecting: false,
            last_reconnect_attempt: std::time::Instant::now(),
            auto_scan_ports: true,
            port_scan_interval_ms: 3000,
            last_port_scan: std::time::Instant::now(),
            logging_enabled: false,
            log_file_path: format!("rustcom_{}.log", Local::now().format("%Y%m%d_%H%M%S")),
            log_entries: Vec::new(),
            filter_enabled: false,
            filter_pattern: String::new(),
            filter_regex: None,
            protocol_mode: ProtocolMode::None,
            virtual_com_enabled: false,
            virtual_com_port: None,
            error_message: None,
            show_advanced: false,
            bytes_received: 0,
            bytes_sent: 0,
        }
    }
}

impl eframe::App for ComAnalyzerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-scan for ports
        if self.auto_scan_ports {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(self.last_port_scan);
            
            if elapsed.as_millis() >= self.port_scan_interval_ms as u128 {
                self.last_port_scan = now;
                
                // Scan for ports and check for changes
                if let Ok(new_ports) = serialport::available_ports() {
                    // Check if ports changed
                    let old_count = self.available_ports.len();
                    let new_count = new_ports.len();
                    
                    // Create a list of port names for comparison
                    let old_names: Vec<String> = self.available_ports.iter()
                        .map(|p| p.port_name.clone())
                        .collect();
                    let new_names: Vec<String> = new_ports.iter()
                        .map(|p| p.port_name.clone())
                        .collect();
                    
                    // Check for new ports
                    for name in &new_names {
                        if !old_names.contains(name) {
                            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                            let msg = format!("[{}] 🔌 New port detected: {}\n", timestamp, name);
                            self.receive_buffer.extend_from_slice(msg.as_bytes());
                        }
                    }
                    
                    // Check for removed ports
                    for name in &old_names {
                        if !new_names.contains(name) {
                            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                            let msg = format!("[{}] 🔌 Port removed: {}\n", timestamp, name);
                            self.receive_buffer.extend_from_slice(msg.as_bytes());
                        }
                    }
                    
                    // Update display if anything changed
                    if old_count != new_count || old_names != new_names {
                        self.available_ports = new_ports;
                        self.update_display_buffer();
                    }
                }
            }
        }
        
        // Read data from serial port if connected
        if self.connected {
            let (read_result, connection_error) = if let Some(port) = &self.serial_port {
                if let Ok(mut port_guard) = port.try_lock() {
                    let mut buffer = vec![0u8; 1024];
                    match port_guard.read(&mut buffer) {
                        Ok(bytes_read) if bytes_read > 0 => {
                            (Some(buffer[..bytes_read].to_vec()), false)
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                            (None, false) // Timeout is normal
                        }
                        Err(_) => {
                            (None, true) // Other errors indicate disconnection
                        }
                        _ => (None, false)
                    }
                } else {
                    (None, false)
                }
            } else {
                (None, false)
            };
            
            // Handle connection error (device disconnected)
            if connection_error {
                let msg = format!("\n[{}] ⚠ Connection lost to {}\n",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    self.selected_port.as_ref().unwrap_or(&"unknown".to_string())
                );
                self.receive_buffer.extend_from_slice(msg.as_bytes());
                self.update_display_buffer();
                
                self.serial_port = None;
                self.connected = false;
                
                if self.auto_reconnect {
                    self.reconnecting = true;
                    self.error_message = Some("Connection lost. Auto-reconnecting...".to_string());
                } else {
                    self.error_message = Some("Connection lost. Device disconnected.".to_string());
                }
            }
            
            // Process read data after releasing the lock
            if let Some(data) = read_result {
                self.bytes_received += data.len();
                
                // Log if enabled
                if self.logging_enabled {
                    self.log_data(Direction::Received, &data);
                }
                
                // Apply filter if enabled
                let should_display = if self.filter_enabled {
                    self.matches_filter(&data)
                } else {
                    true
                };
                
                if should_display {
                    self.receive_buffer.extend_from_slice(&data);
                    self.update_display_buffer();
                }
                
                // Limit buffer size
                if self.receive_buffer.len() > 100000 {
                    self.receive_buffer.drain(0..10000);
                    self.update_display_buffer();
                }
            }
            
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        
        // Handle auto-reconnect
        if self.reconnecting && !self.connected {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(self.last_reconnect_attempt);
            
            if elapsed.as_millis() >= self.reconnect_delay_ms as u128 {
                self.last_reconnect_attempt = now;
                
                // Try to reconnect
                let port_name = self.selected_port.clone();
                if port_name.is_some() {
                    self.connect();
                    
                    if self.connected {
                        self.reconnecting = false;
                        let msg = format!("[{}] ✓ Reconnected successfully\n",
                            Local::now().format("%Y-%m-%d %H:%M:%S")
                        );
                        self.receive_buffer.extend_from_slice(msg.as_bytes());
                        self.update_display_buffer();
                        self.error_message = Some("Reconnected!".to_string());
                    }
                }
            }
            
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        // Top panel with title and stats
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🔌 RustCOM");
                ui.label("|");
                ui.label("Professional COM Port Analyzer");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("📥 {} bytes", self.bytes_received));
                    ui.separator();
                    ui.label(format!("📤 {} bytes", self.bytes_sent));
                });
            });
            ui.separator();
        });

        // Left panel - Connection settings
        egui::SidePanel::left("connection_panel")
            .default_width(280.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(5.0);
                    
                    // Connection section
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("⚙ CONNECTION").strong());
                        ui.separator();
                        
                        ui.horizontal(|ui| {
                            if ui.button("🔄").clicked() {
                                self.available_ports = serialport::available_ports().unwrap_or_default();
                            }
                            ui.label("COM Port:");
                            
                            // Show auto-scan indicator
                            if self.auto_scan_ports {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(egui::RichText::new("⟳").small().color(egui::Color32::GREEN));
                                });
                            }
                        });
                        
                        egui::ComboBox::from_id_source("port_selector")
                            .width(ui.available_width())
                            .selected_text(
                                self.selected_port
                                    .as_ref()
                                    .map(|s| s.as_str())
                                    .unwrap_or("Select port..."),
                            )
                            .show_ui(ui, |ui| {
                                for port in &self.available_ports {
                                    let port_name = port.port_name.clone();
                                    let port_type_str = match &port.port_type {
                                        serialport::SerialPortType::UsbPort(info) => {
                                            format!("(USB {:04x}:{:04x})", info.vid, info.pid)
                                        }
                                        serialport::SerialPortType::PciPort => "(PCI)".to_string(),
                                        serialport::SerialPortType::BluetoothPort => "(BT)".to_string(),
                                        serialport::SerialPortType::Unknown => "".to_string(),
                                    };
                                    let label = format!("{} {}", port_name, port_type_str);
                                    ui.selectable_value(&mut self.selected_port, Some(port_name), label);
                                }
                            });
                        
                        ui.add_space(5.0);
                        
                        ui.horizontal(|ui| {
                            ui.label("Baud:");
                            egui::ComboBox::from_id_source("baud_rate")
                                .width(ui.available_width() - 50.0)
                                .selected_text(&self.baud_rate)
                                .show_ui(ui, |ui| {
                                    for rate in &["300", "1200", "2400", "4800", "9600", "19200", "38400", "57600", "115200", "230400", "460800", "921600"] {
                                        ui.selectable_value(&mut self.baud_rate, rate.to_string(), *rate);
                                    }
                                });
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Data:");
                            egui::ComboBox::from_id_source("data_bits")
                                .width(60.0)
                                .selected_text(self.data_bits.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.data_bits, DataBits::Five, "5");
                                    ui.selectable_value(&mut self.data_bits, DataBits::Six, "6");
                                    ui.selectable_value(&mut self.data_bits, DataBits::Seven, "7");
                                    ui.selectable_value(&mut self.data_bits, DataBits::Eight, "8");
                                });
                            
                            ui.label("Stop:");
                            egui::ComboBox::from_id_source("stop_bits")
                                .width(60.0)
                                .selected_text(self.stop_bits.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.stop_bits, StopBits::One, "1");
                                    ui.selectable_value(&mut self.stop_bits, StopBits::Two, "2");
                                });
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Parity:");
                            egui::ComboBox::from_id_source("parity")
                                .width(ui.available_width() - 55.0)
                                .selected_text(self.parity.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.parity, Parity::None, "None");
                                    ui.selectable_value(&mut self.parity, Parity::Even, "Even");
                                    ui.selectable_value(&mut self.parity, Parity::Odd, "Odd");
                                });
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Flow:");
                            egui::ComboBox::from_id_source("flow_control")
                                .width(ui.available_width() - 45.0)
                                .selected_text(self.flow_control.as_str())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.flow_control, FlowControl::None, "None");
                                    ui.selectable_value(&mut self.flow_control, FlowControl::Software, "Software");
                                    ui.selectable_value(&mut self.flow_control, FlowControl::Hardware, "Hardware");
                                });
                        });
                        
                        ui.add_space(10.0);
                        
                        // Connect button - centered
                        ui.vertical_centered(|ui| {
                            let button_text = if self.connected { "🔌 Disconnect" } else { "⚡ Connect" };
                            let button_color = if self.connected {
                                egui::Color32::from_rgb(200, 60, 60)
                            } else {
                                egui::Color32::from_rgb(60, 160, 60)
                            };
                            
                            let button = egui::Button::new(
                                egui::RichText::new(button_text).size(16.0)
                            )
                            .fill(button_color)
                            .min_size(egui::vec2(220.0, 45.0));
                            
                            if ui.add(button).clicked() {
                                if self.connected {
                                    self.disconnect();
                                } else {
                                    self.connect();
                                }
                            }
                        });
                        
                        ui.add_space(5.0);
                        
                        // Status
                        ui.vertical_centered(|ui| {
                            let (status_text, status_color) = if self.connected {
                                ("● CONNECTED", egui::Color32::GREEN)
                            } else {
                                ("○ DISCONNECTED", egui::Color32::GRAY)
                            };
                            ui.colored_label(status_color, egui::RichText::new(status_text).strong());
                        });
                    });
                    
                    ui.add_space(5.0);
                    
                    // Signal Control
                    if self.connected {
                        ui.group(|ui| {
                            ui.label(egui::RichText::new("📡 SIGNALS").strong());
                            ui.separator();
                            
                            ui.horizontal(|ui| {
                                if ui.checkbox(&mut self.dtr_state, "DTR").changed() {
                                    self.set_dtr(self.dtr_state);
                                }
                                if ui.checkbox(&mut self.rts_state, "RTS").changed() {
                                    self.set_rts(self.rts_state);
                                }
                            });
                        });
                        
                        ui.add_space(5.0);
                    }
                    
                    // View Options
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("👁 VIEW").strong());
                        ui.separator();
                        
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut self.view_mode, ViewMode::Ascii, "ASCII");
                            ui.selectable_value(&mut self.view_mode, ViewMode::Hex, "HEX");
                            ui.selectable_value(&mut self.view_mode, ViewMode::Both, "Both");
                        });
                        
                        ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                        ui.checkbox(&mut self.show_timestamps, "Timestamps");
                        
                        if ui.checkbox(&mut self.strip_ansi, "Strip ANSI codes").changed() {
                            self.update_display_buffer();
                        }
                    });
                    
                    ui.add_space(5.0);
                    
                    // Connection Options
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("🔄 CONNECTION OPTIONS").strong());
                        ui.separator();
                        
                        // Auto-reconnect
                        ui.checkbox(&mut self.auto_reconnect, "Enable auto-reconnect");
                        
                        if self.auto_reconnect {
                            ui.label(egui::RichText::new("Retry if connection is lost").small());
                            
                            ui.horizontal(|ui| {
                                ui.label("Delay:");
                                ui.add(
                                    egui::Slider::new(&mut self.reconnect_delay_ms, 500..=10000)
                                        .suffix("ms")
                                );
                            });
                            
                            if self.reconnecting {
                                ui.colored_label(
                                    egui::Color32::YELLOW,
                                    "⟳ Reconnecting..."
                                );
                            }
                        }
                        
                        ui.add_space(5.0);
                        ui.separator();
                        
                        // Auto-scan ports
                        ui.checkbox(&mut self.auto_scan_ports, "Auto-scan for ports");
                        
                        if self.auto_scan_ports {
                            ui.label(egui::RichText::new("Automatically detect new ports").small());
                            
                            ui.horizontal(|ui| {
                                ui.label("Interval:");
                                ui.add(
                                    egui::Slider::new(&mut self.port_scan_interval_ms, 1000..=10000)
                                        .suffix("ms")
                                );
                            });
                        }
                    });
                    
                    ui.add_space(5.0);
                    
                    // Logging
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("📝 LOGGING").strong());
                        ui.separator();
                        
                        ui.checkbox(&mut self.logging_enabled, "Enable logging");
                        
                        if self.logging_enabled {
                            ui.horizontal(|ui| {
                                ui.label("File:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.log_file_path)
                                        .desired_width(ui.available_width() - 60.0)
                                );
                            });
                            
                            if ui.button("💾 Save Log").clicked() {
                                self.save_log();
                            }
                        }
                    });
                    
                    ui.add_space(5.0);
                    
                    // Filtering
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("🔍 FILTER").strong());
                        ui.separator();
                        
                        ui.checkbox(&mut self.filter_enabled, "Enable filter");
                        
                        if self.filter_enabled {
                            ui.horizontal(|ui| {
                                ui.label("Pattern:");
                            });
                            
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut self.filter_pattern)
                                    .hint_text("regex pattern...")
                            );
                            
                            if response.changed() {
                                self.update_filter();
                            }
                        }
                    });
                    
                    ui.add_space(5.0);
                    
                    // Protocol Analyzer
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("🔬 PROTOCOL").strong());
                        ui.separator();
                        
                        egui::ComboBox::from_id_source("protocol_mode")
                            .width(ui.available_width())
                            .selected_text(self.protocol_mode.as_str())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.protocol_mode, ProtocolMode::None, "None");
                                ui.selectable_value(&mut self.protocol_mode, ProtocolMode::ModbusRTU, "Modbus RTU");
                                ui.selectable_value(&mut self.protocol_mode, ProtocolMode::ModbusASCII, "Modbus ASCII");
                                ui.selectable_value(&mut self.protocol_mode, ProtocolMode::Custom, "Custom");
                            });
                    });
                    
                    ui.add_space(5.0);
                    
                    // Virtual COM
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("🔄 VIRTUAL COM").strong());
                        ui.separator();
                        
                        ui.label(egui::RichText::new("Create loopback COM port pairs").small());
                        
                        if let Some(vport) = &self.virtual_com_port {
                            ui.colored_label(egui::Color32::GREEN, format!("✓ {}", vport));
                            
                            if ui.button("Clear").clicked() {
                                self.virtual_com_port = None;
                                self.virtual_com_enabled = false;
                            }
                        } else {
                            if ui.button("🔧 Create/Find Pair").clicked() {
                                self.create_virtual_com();
                            }
                            
                            ui.label(egui::RichText::new("Click to create or find existing").italics().small());
                            ui.label(egui::RichText::new("com0com port pairs").italics().small());
                        }
                    });
                    
                    // Error display
                    if let Some(error) = &self.error_message {
                        ui.add_space(10.0);
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 100, 100),
                            format!("⚠ {}", error)
                        );
                    }
                });
            });

        // Central panel - Terminal
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                // Terminal header
                ui.horizontal(|ui| {
                    ui.heading("📺 Terminal");
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑 Clear").clicked() {
                            self.receive_buffer.clear();
                            self.receive_buffer_display.clear();
                        }
                        
                        if ui.button("💾 Save").clicked() {
                            self.save_buffer();
                        }
                    });
                });
                
                ui.separator();
                
                // Receive area
                let text_height = ui.available_height() - 80.0;
                
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(self.auto_scroll)
                    .max_height(text_height)
                    .show(ui, |ui| {
                        let display_text = if self.show_timestamps {
                            self.add_timestamps_to_display()
                        } else {
                            self.receive_buffer_display.clone()
                        };
                        
                        ui.add(
                            egui::TextEdit::multiline(&mut display_text.as_str())
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .desired_rows(30)
                        );
                    });
                
                ui.add_space(5.0);
                ui.separator();
                
                // Send area
                ui.horizontal(|ui| {
                    ui.label("Send:");
                    
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.send_buffer)
                            .desired_width(ui.available_width() - 150.0)
                            .hint_text("Type message here...")
                    );
                    
                    let send_clicked = ui.button("📤 Send").clicked();
                    let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    
                    if (send_clicked || enter_pressed) && !self.send_buffer.is_empty() {
                        if self.connected {
                            self.send_data();
                        } else {
                            self.error_message = Some("Not connected".to_string());
                        }
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Options:");
                    ui.checkbox(&mut self.show_advanced, "Show hex input");
                    
                    if self.show_advanced {
                        if ui.button("Send \\r\\n").clicked() && self.connected {
                            self.send_raw(b"\r\n");
                        }
                        if ui.button("Send \\n").clicked() && self.connected {
                            self.send_raw(b"\n");
                        }
                    }
                });
            });
        });
    }
}

impl ComAnalyzerApp {
    fn connect(&mut self) {
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
                let msg = format!("[{}] Connected to {} at {} baud\n", timestamp, port_name, baud_rate);
                self.receive_buffer.extend_from_slice(msg.as_bytes());
                self.update_display_buffer();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to open port: {}", e));
            }
        }
    }

    fn disconnect(&mut self) {
        self.serial_port = None;
        self.connected = false;
        self.reconnecting = false; // Stop auto-reconnect when manually disconnecting
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let msg = format!("[{}] Disconnected\n", timestamp);
        self.receive_buffer.extend_from_slice(msg.as_bytes());
        self.update_display_buffer();
    }

    fn send_data(&mut self) {
        let data = format!("{}\r\n", self.send_buffer);
        let send_result = if let Some(port) = &self.serial_port {
            if let Ok(mut port_guard) = port.lock() {
                port_guard.write(data.as_bytes()).ok()
            } else {
                None
            }
        } else {
            None
        };
        
        // Process result after releasing the lock
        if let Some(bytes) = send_result {
            self.bytes_sent += bytes;
            
            if self.logging_enabled {
                self.log_data(Direction::Sent, data.as_bytes());
            }
            
            let msg = format!("TX: {}\n", self.send_buffer);
            self.receive_buffer.extend_from_slice(msg.as_bytes());
            self.update_display_buffer();
            self.send_buffer.clear();
        } else {
            self.error_message = Some("Send failed".to_string());
        }
    }
    
    fn send_raw(&mut self, data: &[u8]) {
        let data_copy = data.to_vec();
        let send_result = if let Some(port) = &self.serial_port {
            if let Ok(mut port_guard) = port.lock() {
                port_guard.write(&data_copy).ok()
            } else {
                None
            }
        } else {
            None
        };
        
        // Process result after releasing the lock
        if let Some(bytes) = send_result {
            self.bytes_sent += bytes;
            
            if self.logging_enabled {
                self.log_data(Direction::Sent, &data_copy);
            }
        } else {
            self.error_message = Some("Send failed".to_string());
        }
    }
    
    fn set_dtr(&mut self, state: bool) {
        if let Some(port) = &self.serial_port {
            if let Ok(mut port_guard) = port.lock() {
                let _ = port_guard.write_data_terminal_ready(state);
            }
        }
    }
    
    fn set_rts(&mut self, state: bool) {
        if let Some(port) = &self.serial_port {
            if let Ok(mut port_guard) = port.lock() {
                let _ = port_guard.write_request_to_send(state);
            }
        }
    }
    
    fn update_display_buffer(&mut self) {
        self.receive_buffer_display = match self.view_mode {
            ViewMode::Ascii => {
                let raw = String::from_utf8_lossy(&self.receive_buffer).to_string();
                if self.strip_ansi {
                    self.strip_ansi_codes(&raw)
                } else {
                    raw
                }
            },
            ViewMode::Hex => self.format_hex(&self.receive_buffer),
            ViewMode::Both => {
                let ascii_raw = String::from_utf8_lossy(&self.receive_buffer).to_string();
                let ascii = if self.strip_ansi {
                    self.strip_ansi_codes(&ascii_raw)
                } else {
                    ascii_raw
                };
                let hex = self.format_hex(&self.receive_buffer);
                format!("=== HEX ===\n{}\n\n=== ASCII ===\n{}", hex, ascii)
            }
        };
    }
    
    fn strip_ansi_codes(&self, text: &str) -> String {
        // Remove ANSI escape sequences
        // Pattern: ESC [ ... m  or ESC [ ... any letter
        // Common codes: \x1b[0m (reset), \x1b[31m (red), etc.
        
        let mut result = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // Found escape character, check for CSI sequence
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    
                    // Skip everything until we find a letter (command character)
                    while let Some(&next_ch) = chars.peek() {
                        chars.next();
                        if next_ch.is_ascii_alphabetic() {
                            break;
                        }
                    }
                } else {
                    // Other escape sequences (less common)
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }
    
    fn format_hex(&self, data: &[u8]) -> String {
        let mut result = String::new();
        for (i, chunk) in data.chunks(16).enumerate() {
            result.push_str(&format!("{:04X}  ", i * 16));
            
            // Hex bytes
            for (j, byte) in chunk.iter().enumerate() {
                result.push_str(&format!("{:02X} ", byte));
                if j == 7 {
                    result.push(' ');
                }
            }
            
            // Padding
            for _ in 0..(16 - chunk.len()) {
                result.push_str("   ");
            }
            
            result.push_str("  ");
            
            // ASCII representation
            for byte in chunk {
                let ch = if byte.is_ascii_graphic() || *byte == b' ' {
                    *byte as char
                } else {
                    '.'
                };
                result.push(ch);
            }
            
            result.push('\n');
        }
        result
    }
    
    fn log_data(&mut self, direction: Direction, data: &[u8]) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
        let entry = DataLogEntry {
            timestamp,
            direction,
            data: data.to_vec(),
        };
        self.log_entries.push(entry);
    }
    
    fn save_log(&mut self) {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.log_file_path)
        {
            for entry in &self.log_entries {
                let dir_str = match entry.direction {
                    Direction::Received => "RX",
                    Direction::Sent => "TX",
                };
                let data_str = String::from_utf8_lossy(&entry.data);
                let line = format!("[{}] {}: {}\n", entry.timestamp, dir_str, data_str);
                let _ = file.write_all(line.as_bytes());
            }
            self.error_message = Some(format!("Log saved to {}", self.log_file_path));
        } else {
            self.error_message = Some("Failed to save log".to_string());
        }
    }
    
    fn save_buffer(&self) {
        let filename = format!("capture_{}.txt", Local::now().format("%Y%m%d_%H%M%S"));
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .write(true)
            .open(&filename)
        {
            let _ = file.write_all(self.receive_buffer_display.as_bytes());
        }
    }
    
    fn matches_filter(&self, data: &[u8]) -> bool {
        if let Some(regex) = &self.filter_regex {
            let text = String::from_utf8_lossy(data);
            regex.is_match(&text)
        } else {
            true
        }
    }
    
    fn update_filter(&mut self) {
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
    
    fn add_timestamps_to_display(&self) -> String {
        // This is a simplified version - in a full implementation,
        // you'd track timestamps for each received chunk
        self.receive_buffer_display.clone()
    }
    
    fn create_virtual_com(&mut self) {
        match virtual_com::create_loopback_pair() {
            Ok((port1, port2)) => {
                self.virtual_com_port = Some(format!("{} <-> {}", port1, port2));
                self.error_message = Some(format!("Created virtual COM pair: {} and {}", port1, port2));
            }
            Err(e) => {
                self.error_message = Some(e);
            }
        }
    }
}

