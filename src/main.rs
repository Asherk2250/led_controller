mod device;
mod commands;
mod stats;

use device::Device;
use commands::*;
use stats::Stats;
use std::{thread, time::Duration, io::{self, BufRead}};
use std::sync::mpsc;

fn main() {
    // Prompt for COM ports
    println!("Available COM ports: COM3, COM4, COM5, COM6");
    println!("Enter COM port for CPU (e.g., COM3): ");
    
    let stdin = io::stdin();
    let mut cpu_port = String::new();
    stdin.read_line(&mut cpu_port).expect("Failed to read port");
    let cpu_port = cpu_port.trim().to_string();

    println!("Enter COM port for RAM (e.g., COM4): ");
    let mut ram_port = String::new();
    stdin.read_line(&mut ram_port).expect("Failed to read port");
    let ram_port = ram_port.trim().to_string();

    let mut dev_cpu = match Device::connect(&cpu_port) {
        Ok(d) => d,
        Err(e) => {
            println!("Failed to connect to CPU port {}: {}", cpu_port, e);
            return;
        }
    };

    let mut dev_ram = match Device::connect(&ram_port) {
        Ok(d) => d,
        Err(e) => {
            println!("Failed to connect to RAM port {}: {}", ram_port, e);
            return;
        }
    };

    let mut stats = Stats::new();

    dev_cpu.send(brightness(120));
    dev_ram.send(brightness(120));

    // Create channel for user commands
    let (tx, rx) = mpsc::channel();

    // Spawn thread to read user input
    thread::spawn(move || {
        let stdin = io::stdin();
        let reader = stdin.lock();
        for line in reader.lines() {
            if let Ok(cmd) = line {
                let _ = tx.send(cmd);
            }
        }
    });

    println!("CPU display on {}", cpu_port);
    println!("RAM display on {}", ram_port);
    println!("Commands: brightness <0-255>, color <r> <g> <b>, quit");

    loop {
        // Check for user input
        if let Ok(cmd) = rx.try_recv() {
            let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
            
            if !parts.is_empty() {
                match parts[0] {
                    "quit" | "exit" => {
                        println!("Exiting...");
                        break;
                    }
                    "brightness" => {
                        if parts.len() > 1 {
                            if let Ok(level) = parts[1].parse::<u8>() {
                                dev_cpu.send(brightness(level));
                                dev_ram.send(brightness(level));
                                println!("Brightness set to {}", level);
                            }
                        }
                    }
                    "color" => {
                        if parts.len() > 3 {
                            if let (Ok(r), Ok(g), Ok(b)) = (
                                parts[1].parse::<u8>(),
                                parts[2].parse::<u8>(),
                                parts[3].parse::<u8>(),
                            ) {
                                // Create color command (adjust if your protocol differs)
                                let color_cmd = vec![MAGIC1, MAGIC2, 0x13, r, g, b];
                                dev_cpu.send(color_cmd.clone());
                                dev_ram.send(color_cmd);
                                println!("Color set to RGB({}, {}, {})", r, g, b);
                            }
                        }
                    }
                    _ => println!("Unknown command: {}", parts[0]),
                }
            }
        }

        // Update displays - CPU on one device, RAM on the other
        let cpu = stats.cpu_usage();
        let ram = stats.ram_usage();
        dev_cpu.send(percentage(cpu));
        dev_ram.send(percentage(ram));

        thread::sleep(Duration::from_millis(500));
    }
}

