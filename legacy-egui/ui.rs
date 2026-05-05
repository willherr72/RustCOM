use eframe::egui;
use std::time::Duration;
use chrono::Local;

use crate::app::*;
use crate::logging::{self, Direction};
use crate::serial::*;

impl eframe::App for ComAnalyzerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_ports(ctx);
        self.poll_serial(ctx);
        self.poll_reconnect(ctx);

        self.render_top_panel(ctx);
        self.render_side_panel(ctx);
        self.render_central_panel(ctx);
    }
}

impl ComAnalyzerApp {
    fn poll_ports(&mut self, _ctx: &egui::Context) {
        if !self.auto_scan_ports {
            return;
        }

        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_port_scan);

        if elapsed.as_millis() < self.port_scan_interval_ms as u128 {
            return;
        }

        self.last_port_scan = now;

        if let Ok(new_ports) = serialport::available_ports() {
            let old_names: Vec<String> = self.available_ports.iter().map(|p| p.port_name.clone()).collect();
            let new_names: Vec<String> = new_ports.iter().map(|p| p.port_name.clone()).collect();

            for name in &new_names {
                if !old_names.contains(name) {
                    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                    let msg = format!("[{}] New port detected: {}\n", timestamp, name);
                    self.receive_buffer.extend_from_slice(msg.as_bytes());
                }
            }

            for name in &old_names {
                if !new_names.contains(name) {
                    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                    let msg = format!("[{}] Port removed: {}\n", timestamp, name);
                    self.receive_buffer.extend_from_slice(msg.as_bytes());
                }
            }

            if old_names != new_names {
                self.available_ports = new_ports;
                self.update_display_buffer();
            }
        }
    }

    fn poll_serial(&mut self, ctx: &egui::Context) {
        if !self.connected {
            return;
        }

        let (read_result, connection_error) = if let Some(port) = &self.serial_port {
            if let Ok(mut port_guard) = port.try_lock() {
                let mut buffer = vec![0u8; SERIAL_READ_BUFFER_SIZE];
                match port_guard.read(&mut buffer) {
                    Ok(bytes_read) if bytes_read > 0 => {
                        (Some(buffer[..bytes_read].to_vec()), false)
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (None, false),
                    Err(_) => (None, true),
                    _ => (None, false),
                }
            } else {
                (None, false)
            }
        } else {
            (None, false)
        };

        if connection_error {
            let msg = format!(
                "\n[{}] Connection lost to {}\n",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                self.selected_port.as_deref().unwrap_or("unknown")
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

        if let Some(data) = read_result {
            self.bytes_received += data.len();

            if self.logging_enabled {
                self.log_entries
                    .push(logging::create_log_entry(Direction::Received, &data));
            }

            let should_display = if self.filter_enabled {
                self.matches_filter(&data)
            } else {
                true
            };

            if should_display {
                self.receive_buffer.extend_from_slice(&data);
                self.update_display_buffer();
            }

            if self.receive_buffer.len() > MAX_BUFFER_SIZE {
                self.receive_buffer.drain(0..BUFFER_DRAIN_SIZE);
                self.update_display_buffer();
            }
        }

        ctx.request_repaint_after(Duration::from_millis(DEFAULT_REPAINT_INTERVAL_MS));
    }

    fn poll_reconnect(&mut self, ctx: &egui::Context) {
        if !self.reconnecting || self.connected {
            return;
        }

        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_reconnect_attempt);

        if elapsed.as_millis() >= self.reconnect_delay_ms as u128 {
            self.last_reconnect_attempt = now;

            if self.selected_port.is_some() {
                self.connect();

                if self.connected {
                    self.reconnecting = false;
                    let msg = format!(
                        "[{}] Reconnected successfully\n",
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

    fn render_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("RustCOM");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("RX: {} bytes", self.bytes_received));
                    ui.separator();
                    ui.label(format!("TX: {} bytes", self.bytes_sent));
                });
            });

            // Dismissible error bar
            if let Some(error) = self.error_message.clone() {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(255, 100, 100), &error);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("x").clicked() {
                            self.error_message = None;
                        }
                    });
                });
            }

            ui.separator();
        });
    }

    fn render_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("connection_panel")
            .default_width(SIDEBAR_WIDTH)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(5.0);
                    self.render_connection_group(ui);
                    ui.add_space(5.0);

                    if self.connected {
                        self.render_signals_group(ui);
                        ui.add_space(5.0);
                    }

                    self.render_view_group(ui);
                    ui.add_space(5.0);
                    self.render_logging_group(ui);
                    ui.add_space(5.0);
                    self.render_filter_group(ui);
                    ui.add_space(5.0);
                    self.render_virtual_com_group(ui);
                });
            });
    }

    fn render_connection_group(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(egui::RichText::new("Connection").strong());
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Refresh").clicked() {
                    self.available_ports = serialport::available_ports().unwrap_or_default();
                }
                ui.label("COM Port:");

                if self.auto_scan_ports {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("auto")
                                .small()
                                .color(egui::Color32::GREEN),
                        );
                    });
                }
            });

            egui::ComboBox::from_id_source("port_selector")
                .width(ui.available_width())
                .selected_text(
                    self.selected_port
                        .as_deref()
                        .unwrap_or("Select port..."),
                )
                .show_ui(ui, |ui: &mut egui::Ui| {
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
                    .show_ui(ui, |ui: &mut egui::Ui| {
                        for rate in &[
                            "300", "1200", "2400", "4800", "9600", "19200", "38400", "57600",
                            "115200", "230400", "460800", "921600",
                        ] {
                            ui.selectable_value(&mut self.baud_rate, rate.to_string(), *rate);
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Data:");
                egui::ComboBox::from_id_source("data_bits")
                    .width(60.0)
                    .selected_text(self.data_bits.as_str())
                    .show_ui(ui, |ui: &mut egui::Ui| {
                        ui.selectable_value(&mut self.data_bits, DataBits::Five, "5");
                        ui.selectable_value(&mut self.data_bits, DataBits::Six, "6");
                        ui.selectable_value(&mut self.data_bits, DataBits::Seven, "7");
                        ui.selectable_value(&mut self.data_bits, DataBits::Eight, "8");
                    });

                ui.label("Stop:");
                egui::ComboBox::from_id_source("stop_bits")
                    .width(60.0)
                    .selected_text(self.stop_bits.as_str())
                    .show_ui(ui, |ui: &mut egui::Ui| {
                        ui.selectable_value(&mut self.stop_bits, StopBits::One, "1");
                        ui.selectable_value(&mut self.stop_bits, StopBits::Two, "2");
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Parity:");
                egui::ComboBox::from_id_source("parity")
                    .width(ui.available_width() - 55.0)
                    .selected_text(self.parity.as_str())
                    .show_ui(ui, |ui: &mut egui::Ui| {
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
                    .show_ui(ui, |ui: &mut egui::Ui| {
                        ui.selectable_value(&mut self.flow_control, FlowControl::None, "None");
                        ui.selectable_value(
                            &mut self.flow_control,
                            FlowControl::Software,
                            "Software",
                        );
                        ui.selectable_value(
                            &mut self.flow_control,
                            FlowControl::Hardware,
                            "Hardware",
                        );
                    });
            });

            ui.add_space(10.0);

            // Connect button - fill available width
            let button_text = if self.connected {
                "Disconnect"
            } else {
                "Connect"
            };
            let button_color = if self.connected {
                egui::Color32::from_rgb(200, 60, 60)
            } else {
                egui::Color32::from_rgb(60, 160, 60)
            };

            let button_width = ui.available_width();
            let layout = egui::Layout::top_down(egui::Align::Center);
            ui.allocate_ui_with_layout(egui::vec2(button_width, 32.0), layout, |ui| {
                let button = egui::Button::new(egui::RichText::new(button_text))
                    .fill(button_color)
                    .min_size(egui::vec2(button_width, 32.0));

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
                    ("CONNECTED", egui::Color32::GREEN)
                } else {
                    ("DISCONNECTED", egui::Color32::GRAY)
                };
                ui.colored_label(status_color, egui::RichText::new(status_text).strong());
            });

            // Advanced options (collapsible)
            ui.add_space(5.0);
            egui::CollapsingHeader::new("Advanced")
                .default_open(false)
                .show(ui, |ui| {
                    ui.checkbox(&mut self.auto_reconnect, "Auto-reconnect");

                    if self.auto_reconnect {
                        ui.horizontal(|ui| {
                            ui.label("Delay:");
                            ui.add(
                                egui::Slider::new(&mut self.reconnect_delay_ms, 500..=10000)
                                    .suffix("ms"),
                            );
                        });

                        if self.reconnecting {
                            ui.colored_label(egui::Color32::YELLOW, "Reconnecting...");
                        }
                    }

                    ui.checkbox(&mut self.auto_scan_ports, "Auto-scan for ports");

                    if self.auto_scan_ports {
                        ui.horizontal(|ui| {
                            ui.label("Interval:");
                            ui.add(
                                egui::Slider::new(&mut self.port_scan_interval_ms, 1000..=10000)
                                    .suffix("ms"),
                            );
                        });
                    }
                });
        });
    }

    fn render_signals_group(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(egui::RichText::new("Signals").strong());
            ui.separator();

            ui.horizontal(|ui| {
                let dtr_response = ui.checkbox(&mut self.dtr_state, "DTR");
                if dtr_response.changed() {
                    self.set_dtr(self.dtr_state);
                }
                dtr_response.on_hover_text("Data Terminal Ready");

                let rts_response = ui.checkbox(&mut self.rts_state, "RTS");
                if rts_response.changed() {
                    self.set_rts(self.rts_state);
                }
                rts_response.on_hover_text("Request To Send");
            });
        });
    }

    fn render_view_group(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(egui::RichText::new("View").strong());
            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.view_mode, ViewMode::Ascii, "ASCII");
                ui.selectable_value(&mut self.view_mode, ViewMode::Hex, "HEX");
                ui.selectable_value(&mut self.view_mode, ViewMode::Both, "Both");
            });

            ui.checkbox(&mut self.auto_scroll, "Auto-scroll");

            if ui.checkbox(&mut self.strip_ansi, "Strip ANSI codes").changed() {
                self.update_display_buffer();
            }
        });
    }

    fn render_logging_group(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(egui::RichText::new("Logging").strong());
            ui.separator();

            ui.checkbox(&mut self.logging_enabled, "Enable logging");

            if self.logging_enabled {
                ui.horizontal(|ui| {
                    ui.label("File:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.log_file_path)
                            .desired_width(ui.available_width() - 60.0),
                    );
                });

                if ui.button("Save Log").clicked() {
                    self.save_log();
                }
            }
        });
    }

    fn render_filter_group(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(egui::RichText::new("Filter").strong());
            ui.separator();

            ui.checkbox(&mut self.filter_enabled, "Enable filter");

            if self.filter_enabled {
                ui.label("Pattern:");

                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.filter_pattern)
                        .hint_text("regex pattern..."),
                );

                if response.changed() {
                    self.update_filter();
                }
            }
        });
    }

    fn render_virtual_com_group(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(egui::RichText::new("Virtual COM").strong());
            ui.separator();

            ui.label(egui::RichText::new("Create loopback COM port pairs").small());

            if let Some(vport) = &self.virtual_com_port.clone() {
                ui.colored_label(egui::Color32::GREEN, vport);

                if ui.button("Clear").clicked() {
                    self.virtual_com_port = None;
                }
            } else {
                if ui.button("Create/Find Pair").clicked() {
                    self.create_virtual_com();
                }

                ui.label(
                    egui::RichText::new("Click to create or find existing com0com port pairs")
                        .italics()
                        .small(),
                );
            }
        });
    }

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                // Terminal header
                ui.horizontal(|ui| {
                    ui.heading("Terminal");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Clear").clicked() {
                            self.receive_buffer.clear();
                            self.receive_buffer_display.clear();
                        }

                        if ui.button("Save").clicked() {
                            self.save_buffer();
                        }
                    });
                });

                ui.separator();

                // Receive area
                let text_height = ui.available_height() - 60.0;

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(self.auto_scroll)
                    .max_height(text_height)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.receive_buffer_display.as_str())
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .desired_rows(30),
                        );
                    });

                ui.add_space(5.0);
                ui.separator();

                // Send area
                ui.horizontal(|ui| {
                    // Mode toggle
                    ui.selectable_value(&mut self.send_mode, SendMode::Ascii, "ASCII");
                    ui.selectable_value(&mut self.send_mode, SendMode::Hex, "Hex");

                    // Line ending selector (only in ASCII mode)
                    if self.send_mode == SendMode::Ascii {
                        egui::ComboBox::from_id_source("line_ending")
                            .width(50.0)
                            .selected_text(self.line_ending.as_str())
                            .show_ui(ui, |ui: &mut egui::Ui| {
                                ui.selectable_value(
                                    &mut self.line_ending,
                                    LineEnding::None,
                                    "None",
                                );
                                ui.selectable_value(&mut self.line_ending, LineEnding::Cr, "\\r");
                                ui.selectable_value(&mut self.line_ending, LineEnding::Lf, "\\n");
                                ui.selectable_value(
                                    &mut self.line_ending,
                                    LineEnding::CrLf,
                                    "\\r\\n",
                                );
                            });
                    }

                    let hint = match self.send_mode {
                        SendMode::Ascii => "Type message here...",
                        SendMode::Hex => "AA BB 0D 0A ...",
                    };

                    let send_id = egui::Id::new("send_input");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.send_buffer)
                            .id(send_id)
                            .desired_width(ui.available_width() - 60.0)
                            .hint_text(hint),
                    );

                    let send_clicked = ui.button("Send").clicked();
                    let enter_pressed =
                        response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                    if send_clicked || enter_pressed {
                        if self.connected && !self.send_buffer.is_empty() {
                            match self.send_mode {
                                SendMode::Ascii => self.send_data(),
                                SendMode::Hex => self.send_hex_input(),
                            }
                        } else if !self.connected {
                            self.error_message = Some("Not connected".to_string());
                        }
                        // Always re-focus the input after send/enter
                        ui.memory_mut(|mem| mem.request_focus(send_id));
                    }
                });
            });
        });
    }
}
