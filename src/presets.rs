use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

pub const MATRIX_WIDTH: usize = 9;
pub const MATRIX_HEIGHT: usize = 34;
pub const PRESET_FILE: &str = "custom_presets.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomPreset {
    pub name: String,
    pub image_data: Vec<u8>, // 9*34 = 306 pixels, each u8 is brightness 0-255
}

#[derive(Serialize, Deserialize, Default)]
pub struct PresetManager {
    pub presets: HashMap<String, CustomPreset>,
}

impl PresetManager {
    pub fn new() -> Self {
        Self {
            presets: HashMap::new(),
        }
    }

    pub fn load_from_file() -> Self {
        if let Ok(content) = fs::read_to_string(PRESET_FILE) {
            if let Ok(manager) = serde_json::from_str(&content) {
                return manager;
            }
        }
        Self::new()
    }

    pub fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(PRESET_FILE, json)?;
        Ok(())
    }

    pub fn save_preset(&mut self, name: String, image_data: Vec<u8>) -> Result<(), String> {
        if image_data.len() != MATRIX_WIDTH * MATRIX_HEIGHT {
            return Err(format!(
                "Invalid image data size. Expected {}, got {}",
                MATRIX_WIDTH * MATRIX_HEIGHT,
                image_data.len()
            ));
        }

        self.presets.insert(
            name.clone(),
            CustomPreset {
                name: name.clone(),
                image_data,
            },
        );

        self.save_to_file()
            .map_err(|e| format!("Failed to save preset: {}", e))?;
        Ok(())
    }

    pub fn get_preset(&self, name: &str) -> Option<Vec<u8>> {
        self.presets.get(name).map(|p| p.image_data.clone())
    }

    pub fn delete_preset(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.presets.remove(name);
        self.save_to_file()?;
        Ok(())
    }

    pub fn list_presets(&self) -> Vec<String> {
        self.presets.keys().cloned().collect()
    }
}

/// Convert image data to device command bytes for greyscale display
pub fn image_data_to_command(image_data: &[u8]) -> Vec<u8> {
    if image_data.len() != MATRIX_WIDTH * MATRIX_HEIGHT {
        return Vec::new();
    }

    // Send each column as greyscale values
    let mut commands = Vec::new();

    for x in 0..MATRIX_WIDTH {
        // Stage column command: [MAGIC1, MAGIC2, 0x07, x, y0, y1, ..., y33]
        commands.push(0x32); // MAGIC1
        commands.push(0xAC); // MAGIC2
        commands.push(0x07); // StageGreyCol command
        commands.push(x as u8);

        // Add all pixels in this column
        for y in 0..MATRIX_HEIGHT {
            let idx = x + y * MATRIX_WIDTH;
            commands.push(image_data[idx]);
        }
    }

    // Add flush command: [MAGIC1, MAGIC2, 0x08, 0x00]
    commands.push(0x32); // MAGIC1
    commands.push(0xAC); // MAGIC2
    commands.push(0x08); // FlushCols command
    commands.push(0x00);

    commands
}

