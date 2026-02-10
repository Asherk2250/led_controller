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
    cpu_port: String,
    ram_port: String,
    connected: bool,
    devices: Option<(Arc<Mutex<Device>>, Arc<Mutex<Device>>)>,
    stats: Option<Arc<Mutex<Stats>>>,
    cpu_percent: u8,
    ram_percent: u8,
    brightness_level: u8,
    color_r: u8,
    color_g: u8,
    color_b: u8,
    available_ports: Vec<String>,
    last_update: Instant,
    status_message: String,
}

impl Default for MyApp {
    fn default() -> Self {
        let available_ports = get_available_ports();
        Self {
            cpu_port: available_ports.get(0).cloned().unwrap_or_default(),
            ram_port: available_ports.get(1).cloned().unwrap_or_default(),
            connected: false,
            devices: None,
            stats: None,
            cpu_percent: 0,
            ram_percent: 0,
            brightness_level: 120,
            color_r: 255,
            color_g: 255,
            color_b: 255,
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
            
            ui.separator();

            // Connection Status
            let status_color = if self.connected {
                egui::Color32::GREEN
            } else {
                egui::Color32::RED
            };
            ui.colored_label(
                status_color,
                format!(
                    "Status: {}",
                    if self.connected {
                        "Connected"
                    } else {
                        "Disconnected"
                    }
                ),
            );
            ui.label(&self.status_message);

            ui.separator();

            // Port Selection
            if !self.connected {
                ui.horizontal(|ui| {
                    ui.label("CPU Port:");
                    egui::ComboBox::from_id_source("cpu_port")
                        .selected_text(&self.cpu_port)
                        .show_ui(ui, |ui| {
                            for port in &self.available_ports {
                                ui.selectable_value(&mut self.cpu_port, port.clone(), port);
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("RAM Port:");
                    egui::ComboBox::from_id_source("ram_port")
                        .selected_text(&self.ram_port)
                        .show_ui(ui, |ui| {
                            for port in &self.available_ports {
                                ui.selectable_value(&mut self.ram_port, port.clone(), port);
                            }
                        });
                });

                if ui.button("Connect").clicked() {
                    self.connect_devices();
                }
            } else {
                if ui.button("Disconnect").clicked() {
                    self.disconnect_devices();
                }
            }

            ui.separator();

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

            if self.connected {
                // Brightness Control
                ui.group(|ui| {
                    ui.label("Brightness");
                    if ui.add(egui::Slider::new(&mut self.brightness_level, 0..=255)).changed() {
                        self.send_brightness();
                    }
                    ui.label(format!("Level: {}", self.brightness_level));
                });

                ui.separator();

                // Color Control
                ui.group(|ui| {
                    ui.label("Color (RGB)");
                    let mut changed = false;
                    changed |= ui.add(egui::Slider::new(&mut self.color_r, 0..=255)).changed();
                    ui.label(format!("Red: {}", self.color_r));

                    changed |= ui.add(egui::Slider::new(&mut self.color_g, 0..=255)).changed();
                    ui.label(format!("Green: {}", self.color_g));

                    changed |= ui.add(egui::Slider::new(&mut self.color_b, 0..=255)).changed();
                    ui.label(format!("Blue: {}", self.color_b));

                    if changed {
                        self.send_color();
                    }
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
    fn connect_devices(&mut self) {
        match Device::connect(&self.cpu_port) {
            Ok(mut dev) => {
                dev.send(brightness(self.brightness_level));
                let dev_cpu = Arc::new(Mutex::new(dev));

                match Device::connect(&self.ram_port) {
                    Ok(mut dev) => {
                        dev.send(brightness(self.brightness_level));
                        let dev_ram = Arc::new(Mutex::new(dev));

                        let mut stats = Stats::new();
                        stats.refresh();

                        self.devices = Some((dev_cpu, dev_ram));
                        self.stats = Some(Arc::new(Mutex::new(stats)));
                        self.connected = true;
                        self.status_message = format!(
                            "Connected to {} (CPU) and {} (RAM)",
                            self.cpu_port, self.ram_port
                        );
                    }
                    Err(e) => {
                        self.status_message =
                            format!("Failed to connect to RAM port {}: {}", self.ram_port, e);
                    }
                }
            }
            Err(e) => {
                self.status_message =
                    format!("Failed to connect to CPU port {}: {}", self.cpu_port, e);
            }
        }
    }

    fn disconnect_devices(&mut self) {
        self.devices = None;
        self.stats = None;
        self.connected = false;
        self.status_message = "Disconnected".to_string();
    }

    fn update_metrics(&mut self) {
        if let Some(stats_arc) = &self.stats {
            if let Ok(mut stats) = stats_arc.lock() {
                self.cpu_percent = stats.cpu_usage();
                self.ram_percent = stats.ram_usage();

                // Send to devices
                if let Some((dev_cpu, dev_ram)) = &self.devices {
                    if let Ok(mut dev) = dev_cpu.lock() {
                        dev.send(percentage(self.cpu_percent));
                    }
                    if let Ok(mut dev) = dev_ram.lock() {
                        dev.send(percentage(self.ram_percent));
                    }
                }
            }
        }
    }

    fn send_brightness(&mut self) {
        if let Some((dev_cpu, dev_ram)) = &self.devices {
            if let Ok(mut dev) = dev_cpu.lock() {
                dev.send(brightness(self.brightness_level));
            }
            if let Ok(mut dev) = dev_ram.lock() {
                dev.send(brightness(self.brightness_level));
            }
        }
    }

    fn send_color(&mut self) {
        if let Some((dev_cpu, dev_ram)) = &self.devices {
            let color_cmd = vec![MAGIC1, MAGIC2, 0x13, self.color_r, self.color_g, self.color_b];
            if let Ok(mut dev) = dev_cpu.lock() {
                dev.send(color_cmd.clone());
            }
            if let Ok(mut dev) = dev_ram.lock() {
                dev.send(color_cmd);
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


