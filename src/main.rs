mod device;
mod commands;
mod stats;
mod presets;
mod utils;

use device::Device;
use commands::*;
use stats::Stats;
use presets::{PresetManager, MATRIX_WIDTH, MATRIX_HEIGHT, image_data_to_command};
use utils::*;
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
    left_preset: String,
    right_preset: String,
    left_brightness: u8,
    right_brightness: u8,
    available_ports: Vec<String>,
    last_update: Instant,
    idle_frame: u8,
    status_message: String,
    // Image editor fields
    editor_image: Vec<u8>,
    editor_brightness: u8,
    editor_preset_name: String,
    preset_manager: PresetManager,
    selected_custom_preset: Option<String>,
    show_editor: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        let available_ports = get_available_ports();
        let preset_manager = PresetManager::load_from_file();
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
            left_preset: "idle".to_string(),
            right_preset: "idle".to_string(),
            left_brightness: 120,
            right_brightness: 120,
            available_ports,
            last_update: Instant::now(),
            idle_frame: 0,
            status_message: "Ready to connect".to_string(),
            editor_image: vec![0u8; MATRIX_WIDTH * MATRIX_HEIGHT],
            editor_brightness: 255,
            editor_preset_name: String::new(),
            preset_manager,
            selected_custom_preset: None,
            show_editor: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Framework LED Controller");
            
            // Show connection status for both sides
            let left_status_text = if self.left_connected { "✓ Connected" } else { "✗ Disconnected" };
            let right_status_text = if self.right_connected { "✓ Connected" } else { "✗ Disconnected" };
            
            ui.horizontal(|ui| {
                ui.label("Left:");
                ui.colored_label(
                    if self.left_connected { egui::Color32::GREEN } else { egui::Color32::RED },
                    left_status_text
                );
                ui.label("  |  Right:");
                ui.colored_label(
                    if self.right_connected { egui::Color32::GREEN } else { egui::Color32::RED },
                    right_status_text
                );
            });
            
            ui.label(&self.status_message);
            ui.separator();
            
            // Main scrollable area with left and right columns
            egui::ScrollArea::both().auto_shrink([false; 2]).show(ui, |ui| {
                ui.horizontal(|ui| {
                    // LEFT SECTION
                    ui.vertical(|ui| {
                        ui.set_width(300.0);

                        ui.group(|ui| {
                            ui.heading("⬅️ Left Matrix");
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
                                
                                // Left Side Settings
                                ui.label("Brightness:");
                                if ui.add(egui::Slider::new(&mut self.left_brightness, 0..=255).step_by(1.0)).changed() {
                                    self.send_left_brightness();
                                }
                                ui.label(format!("Level: {}", self.left_brightness));
                                
                                ui.label("Preset:");
                                egui::ComboBox::from_id_source("left_preset")
                                    .selected_text(&self.left_preset)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.left_preset, "idle".to_string(), "Idle Animation");
                                        ui.separator();
                                        ui.label("📊 System Metrics");
                                        ui.selectable_value(&mut self.left_preset, "cpu".to_string(), "  CPU Usage");
                                        ui.selectable_value(&mut self.left_preset, "ram".to_string(), "  RAM Usage");
                                        ui.separator();
                                        ui.label("⏰ Display");
                                        ui.selectable_value(&mut self.left_preset, "clock".to_string(), "  Clock");
                                        ui.selectable_value(&mut self.left_preset, "battery".to_string(), "  Battery");
                                        ui.separator();
                                        ui.label("🎨 Patterns");
                                        ui.selectable_value(&mut self.left_preset, "gradient".to_string(), "  Gradient");
                                        ui.selectable_value(&mut self.left_preset, "double_gradient".to_string(), "  Double Gradient");
                                        ui.selectable_value(&mut self.left_preset, "zigzag".to_string(), "  ZigZag");
                                        ui.selectable_value(&mut self.left_preset, "lotus_h".to_string(), "  LOTUS Horiz");
                                        ui.selectable_value(&mut self.left_preset, "lotus_v".to_string(), "  LOTUS Vert");
                                        ui.selectable_value(&mut self.left_preset, "full_brightness".to_string(), "  Full Bright");
                                        ui.selectable_value(&mut self.left_preset, "panic".to_string(), "  ⚠️ PANIC");
                                        ui.separator();
                                        ui.label("🖼️ Custom Presets");
                                        for preset_name in self.preset_manager.list_presets() {
                                            ui.selectable_value(&mut self.left_preset, preset_name.clone(), format!("  {}", preset_name));
                                        }
                                    });
                            }
                        });
                    });

                    ui.separator();

                    // RIGHT SECTION
                    ui.vertical(|ui| {
                        ui.set_width(300.0);

                        ui.group(|ui| {
                            ui.heading("➡️ Right Matrix");
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
                                
                                // Right Side Settings
                                ui.label("Brightness:");
                                if ui.add(egui::Slider::new(&mut self.right_brightness, 0..=255).step_by(1.0)).changed() {
                                    self.send_right_brightness();
                                }
                                ui.label(format!("Level: {}", self.right_brightness));
                                
                                ui.label("Preset:");
                                egui::ComboBox::from_id_source("right_preset")
                                    .selected_text(&self.right_preset)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut self.right_preset, "idle".to_string(), "Idle Animation");
                                        ui.separator();
                                        ui.label("📊 System Metrics");
                                        ui.selectable_value(&mut self.right_preset, "cpu".to_string(), "  CPU Usage");
                                        ui.selectable_value(&mut self.right_preset, "ram".to_string(), "  RAM Usage");
                                        ui.separator();
                                        ui.label("⏰ Display");
                                        ui.selectable_value(&mut self.right_preset, "clock".to_string(), "  Clock");
                                        ui.selectable_value(&mut self.right_preset, "battery".to_string(), "  Battery");
                                        ui.separator();
                                        ui.label("🎨 Patterns");
                                        ui.selectable_value(&mut self.right_preset, "gradient".to_string(), "  Gradient");
                                        ui.selectable_value(&mut self.right_preset, "double_gradient".to_string(), "  Double Gradient");
                                        ui.selectable_value(&mut self.right_preset, "zigzag".to_string(), "  ZigZag");
                                        ui.selectable_value(&mut self.right_preset, "lotus_h".to_string(), "  LOTUS Horiz");
                                        ui.selectable_value(&mut self.right_preset, "lotus_v".to_string(), "  LOTUS Vert");
                                        ui.selectable_value(&mut self.right_preset, "full_brightness".to_string(), "  Full Bright");
                                        ui.selectable_value(&mut self.right_preset, "panic".to_string(), "  ⚠️ PANIC");
                                        ui.separator();
                                        ui.label("🖼️ Custom Presets");
                                        for preset_name in self.preset_manager.list_presets() {
                                            ui.selectable_value(&mut self.right_preset, preset_name.clone(), format!("  {}", preset_name));
                                        }
                                    });
                            }
                        });
                    });
                });
            });

            ui.group(|ui| {
                ui.heading("🖼️ Image Editor");
                if ui.button("Toggle Editor").clicked() {
                    self.show_editor = !self.show_editor;
                }

                if self.show_editor {
                    // Brightness slider
                    ui.label("Brush Brightness:");
                    ui.add(egui::Slider::new(&mut self.editor_brightness, 0..=255));

                    // Clear and Fill buttons
                    ui.horizontal(|ui| {
                        if ui.button("Clear All").clicked() {
                            self.editor_image = vec![0u8; MATRIX_WIDTH * MATRIX_HEIGHT];
                        }
                        if ui.button("Fill All").clicked() {
                            self.editor_image = vec![self.editor_brightness; MATRIX_WIDTH * MATRIX_HEIGHT];
                        }
                    });

                    // Pixel grid
                    ui.label("Click pixels to draw (9 x 34 grid):");
                    let pixel_size = 12.0;
                    let grid_width = pixel_size * MATRIX_WIDTH as f32;
                    let (response, painter) = ui.allocate_painter(
                        egui::Vec2::new(grid_width, pixel_size * MATRIX_HEIGHT as f32),
                        egui::Sense::click(),
                    );

                    // Draw grid and handle clicks
                    for x in 0..MATRIX_WIDTH {
                        for y in 0..MATRIX_HEIGHT {
                            let rect = egui::Rect::from_min_size(
                                response.rect.min + egui::Vec2::new(x as f32 * pixel_size, y as f32 * pixel_size),
                                egui::Vec2::splat(pixel_size),
                            );

                            let idx = x + y * MATRIX_WIDTH;
                            let brightness = self.editor_image[idx];
                            let color = egui::Color32::from_gray(brightness);

                            painter.rect_filled(rect, 0.0, color);
                            painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, egui::Color32::GRAY));

                            // Handle clicks
                            if response.clicked() {
                                if let Some(pos) = response.interact_pointer_pos() {
                                    let rel_pos = pos - response.rect.min;
                                    let click_x = (rel_pos.x / pixel_size) as usize;
                                    let click_y = (rel_pos.y / pixel_size) as usize;

                                    if click_x == x && click_y == y {
                                        self.editor_image[idx] = self.editor_brightness;
                                    }
                                }
                            }
                        }
                    }

                    ui.separator();

                    // Preset name input and save
                    ui.horizontal(|ui| {
                        ui.label("Preset Name:");
                        ui.text_edit_singleline(&mut self.editor_preset_name);
                    });

                    if ui.button("Save Preset").clicked() {
                        if !self.editor_preset_name.is_empty() {
                            match self.preset_manager.save_preset(
                                self.editor_preset_name.clone(),
                                self.editor_image.clone(),
                            ) {
                                Ok(_) => {
                                    self.status_message = format!("Preset '{}' saved!", self.editor_preset_name);
                                    self.editor_preset_name.clear();
                                }
                                Err(e) => {
                                    self.status_message = format!("Error saving preset: {}", e);
                                }
                            }
                        }
                    }

                    // Load preset
                    ui.label("Load Preset:");
                    let preset_list = self.preset_manager.list_presets();
                    egui::ComboBox::from_label("Select to load")
                        .selected_text(self.selected_custom_preset.clone().unwrap_or_else(|| "None".to_string()))
                        .show_ui(ui, |ui| {
                            for preset in preset_list {
                                if ui.selectable_value(
                                    &mut self.selected_custom_preset,
                                    Some(preset.clone()),
                                    &preset,
                                ).clicked() {
                                    if let Some(data) = self.preset_manager.get_preset(&preset) {
                                        self.editor_image = data;
                                        self.status_message = format!("Loaded preset '{}'", preset);
                                    }
                                }
                            }
                        });

                    // Delete preset
                    if let Some(preset_name) = &self.selected_custom_preset {
                        if ui.button("Delete Preset").clicked() {
                            let _ = self.preset_manager.delete_preset(preset_name);
                            self.status_message = format!("Deleted preset '{}'", preset_name);
                            self.selected_custom_preset = None;
                        }
                    }

                    // Preview on both sides
                    if ui.button("Preview on Left").clicked() && self.left_connected {
                        if let Some(left_dev) = &self.left_device {
                            if let Ok(mut dev) = left_dev.lock() {
                                let command = image_data_to_command(&self.editor_image);
                                dev.send(command);
                            }
                        }
                    }

                    if ui.button("Preview on Right").clicked() && self.right_connected {
                        if let Some(right_dev) = &self.right_device {
                            if let Ok(mut dev) = right_dev.lock() {
                                let command = image_data_to_command(&self.editor_image);
                                dev.send(command);
                            }
                        }
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
                dev.send(brightness(self.left_brightness));
                self.left_device = Some(Arc::new(Mutex::new(dev)));
                self.left_connected = true;
                self.status_message = format!("Left connected to {}", self.left_port);

                // Initialize stats if this is the first connection
                if self.stats.is_none() {
                    let mut stats = Stats::new();
                    stats.refresh();
                    self.stats = Some(Arc::new(Mutex::new(stats)));
                }
            }
            Err(e) => {
                self.status_message = format!("Failed to connect left: {}", e);
            }
        }
    }

    fn connect_right(&mut self) {
        match Device::connect(&self.right_port) {
            Ok(mut dev) => {
                dev.send(brightness(self.right_brightness));
                self.right_device = Some(Arc::new(Mutex::new(dev)));
                self.right_connected = true;
                self.status_message = format!("Right connected to {}", self.right_port);

                // Initialize stats if this is the first connection
                if self.stats.is_none() {
                    let mut stats = Stats::new();
                    stats.refresh();
                    self.stats = Some(Arc::new(Mutex::new(stats)));
                }
            }
            Err(e) => {
                self.status_message = format!("Failed to connect right: {}", e);
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
        // Increment animation frame
        self.idle_frame = self.idle_frame.wrapping_add(1);
        
        if let Some(stats_arc) = &self.stats {
            if let Ok(mut stats) = stats_arc.lock() {
                self.cpu_percent = stats.cpu_usage();
                self.ram_percent = stats.ram_usage();

                // Send commands to left device based on left preset
                if let Some(left_dev) = &self.left_device {
                    if let Ok(mut dev) = left_dev.lock() {
                        let command = match self.left_preset.as_str() {
                            "cpu" => pattern_percentage(self.cpu_percent),
                            "ram" => pattern_percentage(self.ram_percent),
                            "idle" => {
                                let pattern = (self.idle_frame / 4) % 3;
                                vec![MAGIC1, MAGIC2, 0x14, pattern]
                            }
                            "clock" => {
                                let image_data = render_clock_display();
                                image_data_to_command(&image_data)
                            }
                            "battery" => {
                                let image_data = render_battery_display();
                                image_data_to_command(&image_data)
                            }
                            "gradient" => pattern_gradient(),
                            "double_gradient" => pattern_double_gradient(),
                            "zigzag" => pattern_zigzag(),
                            "lotus_h" => pattern_lotus_horizontal(),
                            "lotus_v" => pattern_lotus_vertical(),
                            "full_brightness" => pattern_full_brightness(),
                            "panic" => pattern_panic(),
                            _ => {
                                // Check if it's a custom preset
                                if let Some(image_data) = self.preset_manager.get_preset(&self.left_preset) {
                                    image_data_to_command(&image_data)
                                } else {
                                    Vec::new()
                                }
                            }
                        };
                        
                        if !command.is_empty() {
                            dev.send(command);
                        }
                    }
                }

                // Send commands to right device based on right preset
                if let Some(right_dev) = &self.right_device {
                    if let Ok(mut dev) = right_dev.lock() {
                        let command = match self.right_preset.as_str() {
                            "cpu" => pattern_percentage(self.cpu_percent),
                            "ram" => pattern_percentage(self.ram_percent),
                            "idle" => {
                                let pattern = (self.idle_frame / 4) % 3;
                                vec![MAGIC1, MAGIC2, 0x14, pattern]
                            }
                            "clock" => {
                                let image_data = render_clock_display();
                                image_data_to_command(&image_data)
                            }
                            "battery" => {
                                let image_data = render_battery_display();
                                image_data_to_command(&image_data)
                            }
                            "gradient" => pattern_gradient(),
                            "double_gradient" => pattern_double_gradient(),
                            "zigzag" => pattern_zigzag(),
                            "lotus_h" => pattern_lotus_horizontal(),
                            "lotus_v" => pattern_lotus_vertical(),
                            "full_brightness" => pattern_full_brightness(),
                            "panic" => pattern_panic(),
                            _ => {
                                // Check if it's a custom preset
                                if let Some(image_data) = self.preset_manager.get_preset(&self.right_preset) {
                                    image_data_to_command(&image_data)
                                } else {
                                    Vec::new()
                                }
                            }
                        };
                        
                        if !command.is_empty() {
                            dev.send(command);
                        }
                    }
                }
            }
        }
    }

    fn send_left_brightness(&mut self) {
        if let Some(left_dev) = &self.left_device {
            if let Ok(mut dev) = left_dev.lock() {
                dev.send(brightness(self.left_brightness));
            }
        }
    }

    fn send_right_brightness(&mut self) {
        if let Some(right_dev) = &self.right_device {
            if let Ok(mut dev) = right_dev.lock() {
                dev.send(brightness(self.right_brightness));
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

// Command generators
fn pattern_percentage(value: u8) -> Vec<u8> {
    vec![MAGIC1, MAGIC2, 0x01, 0x00, value]
}

fn pattern_gradient() -> Vec<u8> {
    vec![MAGIC1, MAGIC2, 0x01, 0x01]
}

fn pattern_double_gradient() -> Vec<u8> {
    vec![MAGIC1, MAGIC2, 0x01, 0x02]
}

fn pattern_lotus_horizontal() -> Vec<u8> {
    vec![MAGIC1, MAGIC2, 0x01, 0x03]
}

fn pattern_zigzag() -> Vec<u8> {
    vec![MAGIC1, MAGIC2, 0x01, 0x04]
}

fn pattern_full_brightness() -> Vec<u8> {
    vec![MAGIC1, MAGIC2, 0x01, 0x05]
}

fn pattern_panic() -> Vec<u8> {
    vec![MAGIC1, MAGIC2, 0x01, 0x06]
}

fn pattern_lotus_vertical() -> Vec<u8> {
    vec![MAGIC1, MAGIC2, 0x01, 0x07]
}

fn set_animate(enabled: bool) -> Vec<u8> {
    vec![MAGIC1, MAGIC2, 0x04, if enabled { 1 } else { 0 }]
}


