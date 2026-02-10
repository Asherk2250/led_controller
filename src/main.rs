mod device;
mod commands;
mod stats;

use device::Device;
use commands::*;
use stats::Stats;
use std::{sync::Arc, sync::Mutex};
use std::time::Instant;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Framework LED Controller",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

struct MyApp {
    left_port: String,
    right_port: String,
    left_connected: bool,
    right_connected: bool,
    left_device: Option<Arc<Mutex<Device>>>,
    right_device: Option<Arc<Mutex<Device>>>,
    stats: Option<Arc<Mutex<Stats>>>,
    cpu_percent: u8,
    ram_percent: u8,
    brightness_level: u8,
    current_preset: String,
    available_ports: Vec<String>,
    last_update: Instant,
    status_message: String,
}

impl Default for MyApp {
    fn default() -> Self {
        let available_ports = get_available_ports();
        Self {
            left_port: available_ports.get(0).cloned().unwrap_or_default(),
            right_port: available_ports.get(1).cloned().unwrap_or_default(),
            left_connected: false,
            right_connected: false,
            left_device: None,
            right_device: None,
            stats: None,
            cpu_percent: 0,
            ram_percent: 0,
            brightness_level: 120,
            current_preset: "idle".to_string(),
            available_ports,
            last_update: Instant::now(),
            status_message: "Ready to connect".to_string(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Framework LED Controller");
            ui.label(&self.status_message);
            ui.separator();

            // Left Matrix Section
            ui.group(|ui| {
                ui.heading("Left Matrix");
                let left_status = if self.left_connected { "Connected" } else { "Disconnected" };
                ui.colored_label(
                    if self.left_connected { egui::Color32::GREEN } else { egui::Color32::RED },
                    format!("Status: {}", left_status)
                );

                if !self.left_connected {
                    ui.horizontal(|ui| {
                        ui.label("Port:");
                        egui::ComboBox::from_id_source("left_port")
                            .selected_text(&self.left_port)
                            .show_ui(ui, |ui| {
                                for port in &self.available_ports {
                                    ui.selectable_value(&mut self.left_port, port.clone(), port);
                                }
                            });
                    });
                    if ui.button("Connect Left").clicked() {
                        self.connect_left();
                    }
                } else {
                    if ui.button("Disconnect Left").clicked() {
                        self.disconnect_left();
                    }
                }
            });

            ui.separator();

            // Right Matrix Section
            ui.group(|ui| {
                ui.heading("Right Matrix");
                let right_status = if self.right_connected { "Connected" } else { "Disconnected" };
                ui.colored_label(
                    if self.right_connected { egui::Color32::GREEN } else { egui::Color32::RED },
                    format!("Status: {}", right_status)
                );

                if !self.right_connected {
                    ui.horizontal(|ui| {
                        ui.label("Port:");
                        egui::ComboBox::from_id_source("right_port")
                            .selected_text(&self.right_port)
                            .show_ui(ui, |ui| {
                                for port in &self.available_ports {
                                    ui.selectable_value(&mut self.right_port, port.clone(), port);
                                }
                            });
                    });
                    if ui.button("Connect Right").clicked() {
                        self.connect_right();
                    }
                } else {
                    if ui.button("Disconnect Right").clicked() {
                        self.disconnect_right();
                    }
                }
            });

            ui.separator();

            if self.left_connected || self.right_connected {
                // Display Metrics
                ui.group(|ui| {
                    ui.label("System Metrics");
                    ui.horizontal(|ui| {
                        ui.label(format!("CPU Usage: {}%", self.cpu_percent));
                        ui.add(
                            egui::ProgressBar::new(self.cpu_percent as f32 / 100.0)
                                .text("CPU"),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("RAM Usage: {}%", self.ram_percent));
                        ui.add(
                            egui::ProgressBar::new(self.ram_percent as f32 / 100.0)
                                .text("RAM"),
                        );
                    });
                });

                ui.separator();

                // Brightness Control
                ui.group(|ui| {
                    ui.label("Brightness");
                    if ui.add(egui::Slider::new(&mut self.brightness_level, 0..=255)).changed() {
                        self.send_brightness();
                    }
                    ui.label(format!("Level: {}", self.brightness_level));
                });

                ui.separator();

                // Preset Selection
                ui.group(|ui| {
                    ui.label("Preset");
                    egui::ComboBox::from_id_source("preset")
                        .selected_text(&self.current_preset)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.current_preset, "idle".to_string(), "Idle Animation");
                            ui.selectable_value(&mut self.current_preset, "cpu".to_string(), "CPU Usage");
                            ui.selectable_value(&mut self.current_preset, "ram".to_string(), "RAM Usage");
                        });
                });

                ui.separator();

                // Update metrics
                if self.last_update.elapsed().as_millis() > 500 {
                    self.update_metrics();
                    self.last_update = Instant::now();
                }

                // Request repaint frequently
                ctx.request_repaint();
            }
        });
    }
}

impl MyApp {
    fn connect_left(&mut self) {
        match Device::connect(&self.left_port) {
            Ok(mut dev) => {
                dev.send(brightness(self.brightness_level));
                self.left_device = Some(Arc::new(Mutex::new(dev)));
                self.left_connected = true;
                self.status_message = format!("Left matrix connected to {}", self.left_port);

                // Initialize stats if this is the first connection
                if self.stats.is_none() {
                    let mut stats = Stats::new();
                    stats.refresh();
                    self.stats = Some(Arc::new(Mutex::new(stats)));
                }
            }
            Err(e) => {
                self.status_message = format!("Failed to connect left to {}: {}", self.left_port, e);
            }
        }
    }

    fn connect_right(&mut self) {
        match Device::connect(&self.right_port) {
            Ok(mut dev) => {
                dev.send(brightness(self.brightness_level));
                self.right_device = Some(Arc::new(Mutex::new(dev)));
                self.right_connected = true;
                self.status_message = format!("Right matrix connected to {}", self.right_port);

                // Initialize stats if this is the first connection
                if self.stats.is_none() {
                    let mut stats = Stats::new();
                    stats.refresh();
                    self.stats = Some(Arc::new(Mutex::new(stats)));
                }
            }
            Err(e) => {
                self.status_message = format!("Failed to connect right to {}: {}", self.right_port, e);
            }
        }
    }

    fn disconnect_left(&mut self) {
        self.left_device = None;
        self.left_connected = false;
        if !self.right_connected {
            self.stats = None;
        }
        self.status_message = "Left matrix disconnected".to_string();
    }

    fn disconnect_right(&mut self) {
        self.right_device = None;
        self.right_connected = false;
        if !self.left_connected {
            self.stats = None;
        }
        self.status_message = "Right matrix disconnected".to_string();
    }

    fn update_metrics(&mut self) {
        if let Some(stats_arc) = &self.stats {
            if let Ok(mut stats) = stats_arc.lock() {
                self.cpu_percent = stats.cpu_usage();
                self.ram_percent = stats.ram_usage();

                // Send commands based on preset
                match self.current_preset.as_str() {
                    "cpu" => {
                        if let Some(left_dev) = &self.left_device {
                            if let Ok(mut dev) = left_dev.lock() {
                                dev.send(percentage(self.cpu_percent));
                            }
                        }
                        if let Some(right_dev) = &self.right_device {
                            if let Ok(mut dev) = right_dev.lock() {
                                dev.send(percentage(self.cpu_percent));
                            }
                        }
                    }
                    "ram" => {
                        if let Some(left_dev) = &self.left_device {
                            if let Ok(mut dev) = left_dev.lock() {
                                dev.send(percentage(self.ram_percent));
                            }
                        }
                        if let Some(right_dev) = &self.right_device {
                            if let Ok(mut dev) = right_dev.lock() {
                                dev.send(percentage(self.ram_percent));
                            }
                        }
                    }
                    "idle" => {
                        // Idle animation - you can add a command for this
                        // For now, we'll just send a neutral command
                        if let Some(left_dev) = &self.left_device {
                            if let Ok(mut dev) = left_dev.lock() {
                                dev.send(vec![MAGIC1, MAGIC2, 0x14]); // Idle animation command
                            }
                        }
                        if let Some(right_dev) = &self.right_device {
                            if let Ok(mut dev) = right_dev.lock() {
                                dev.send(vec![MAGIC1, MAGIC2, 0x14]); // Idle animation command
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn send_brightness(&mut self) {
        if let Some(left_dev) = &self.left_device {
            if let Ok(mut dev) = left_dev.lock() {
                dev.send(brightness(self.brightness_level));
            }
        }
        if let Some(right_dev) = &self.right_device {
            if let Ok(mut dev) = right_dev.lock() {
                dev.send(brightness(self.brightness_level));
            }
        }
    }
}

fn get_available_ports() -> Vec<String> {
    serialport::available_ports()
        .map(|ports| {
            ports
                .iter()
                .filter_map(|p| match p {
                    serialport::SerialPortInfo {
                        port_name,
                        port_type: _,
                    } => Some(port_name.clone()),
                })
                .collect()
        })
        .unwrap_or_default()
}


