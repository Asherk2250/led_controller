use chrono::{Local, Timelike};

pub const MATRIX_WIDTH: usize = 9;
pub const MATRIX_HEIGHT: usize = 34;

/// Generate a simple clock display pattern as brightness values
/// Shows HH:MM using a simple pixel arrangement
pub fn render_clock_display() -> Vec<u8> {
    let now = Local::now();
    let hours = now.hour() as u8;
    let minutes = now.minute() as u8;

    let mut image_data = vec![0u8; MATRIX_WIDTH * MATRIX_HEIGHT];

    // Display hours in top section (binary representation)
    render_binary_number(&mut image_data, hours, 0, 0);
    // Display minutes in bottom section (binary representation)
    render_binary_number(&mut image_data, minutes, 0, 17);

    image_data
}

/// Generate a battery display pattern as brightness values
/// Shows battery percentage as a bar
pub fn render_battery_display() -> Vec<u8> {
    let mut image_data = vec![0u8; MATRIX_WIDTH * MATRIX_HEIGHT];
    
    // Try to get battery percentage
    let percent = get_battery_percentage().unwrap_or(100.0);
    render_battery_bar(&mut image_data, percent as u8);
    
    // Also display percentage as binary number
    render_binary_number(&mut image_data, percent as u8, 0, 8);

    image_data
}

/// Get battery percentage from the system
fn get_battery_percentage() -> Option<f32> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        
        // On Windows, try to get battery info
        if let Ok(output) = Command::new("powershell")
            .args(&["-Command", "Get-CimInstance -ClassName Win32_Battery | Select-Object -ExpandProperty EstimatedChargeRemaining"])
            .output()
        {
            if let Ok(text) = String::from_utf8(output.stdout) {
                if let Ok(percent) = text.trim().parse::<f32>() {
                    return Some(percent);
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // On Linux/Mac, try battery crate as fallback
        if let Ok(manager) = battery::Manager::new() {
            if let Ok(batteries) = manager.batteries() {
                for battery in batteries {
                    if let Ok(bat) = battery {
                        let percent = (bat.energy().as_f32() / bat.energy_full().as_f32()) * 100.0;
                        return Some(percent);
                    }
                }
            }
        }
    }
    
    None
}

/// Render a horizontal battery bar at the top of the display
fn render_battery_bar(image_data: &mut [u8], percentage: u8) {
    let percentage = percentage.min(100);
    let filled_rows = ((percentage as usize * MATRIX_HEIGHT) / 100).min(MATRIX_HEIGHT);
    
    // Draw vertical battery indicator on the left side
    for row in 0..MATRIX_HEIGHT {
        for col in 0..2 {
            let idx = col + row * MATRIX_WIDTH;
            if MATRIX_HEIGHT - row <= filled_rows {
                image_data[idx] = if percentage > 50 { 100 } else if percentage > 20 { 150 } else { 255 }; // Green for high, yellow for medium, red for low
            } else {
                image_data[idx] = 20; // Dim for empty portion
            }
        }
    }
}

/// Render a number in binary format (8 bits vertical)
fn render_binary_number(image_data: &mut [u8], number: u8, col_start: usize, row_start: usize) {
    for bit in 0..8 {
        let is_set = (number & (1 << bit)) != 0;
        let brightness = if is_set { 200 } else { 30 };
        let row = row_start + (7 - bit); // MSB at top
        if row < MATRIX_HEIGHT && col_start + 1 < MATRIX_WIDTH {
            let idx = col_start + row * MATRIX_WIDTH;
            image_data[idx] = brightness;
            image_data[idx + 1] = brightness;
        }
    }
}

/// Generate breathing animation pattern
pub fn render_breathing_animation(frame: u8) -> Vec<u8> {
    let mut image_data = vec![0u8; MATRIX_WIDTH * MATRIX_HEIGHT];
    
    // Create a breathing effect - brightness changes with frame
    let brightness = if frame < 128 {
        (frame as f32 / 128.0 * 255.0) as u8
    } else {
        (255.0 - (frame as f32 - 128.0) / 128.0 * 255.0) as u8
    };

    // Fill entire display with breathing brightness
    for pixel in image_data.iter_mut() {
        *pixel = brightness;
    }

    image_data
}
