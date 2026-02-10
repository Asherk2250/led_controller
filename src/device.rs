// src/device.rs
use std::io::Write;
use std::time::Duration;
use serialport::SerialPort;

pub struct Device {
    port: Box<dyn SerialPort>,
}

impl Device {
    pub fn connect(port_name: &str) -> std::io::Result<Self> {
        let port = serialport::new(port_name, 115200)
            .timeout(Duration::from_millis(100))
            .open()?;

        Ok(Device { port })
    }

    pub fn send(&mut self, data: Vec<u8>) {
        let _ = self.port.write_all(&data);
    }
}

use crate::commands::*;

pub struct LedDevice {
    port: Box<dyn serialport::SerialPort>,
}

impl LedDevice {
    pub fn connect(port_name: &str) -> anyhow::Result<Self> {
        let port = serialport::new(port_name, 115200)
            .timeout(Duration::from_millis(200))
            .open()?;
        Ok(Self { port })
    }

    fn send(&mut self, cmd: u8, params: &[u8]) -> anyhow::Result<()> {
        let mut packet = vec![MAGIC1, MAGIC2, cmd];
        packet.extend_from_slice(params);
        self.port.write_all(&packet)?;
        Ok(())
    }

    pub fn set_brightness(&mut self, value: u8) -> anyhow::Result<()> {
        self.send(CMD_BRIGHTNESS, &[value])
    }

    pub fn set_pattern(&mut self, pattern: u8) -> anyhow::Result<()> {
        self.send(CMD_PATTERN, &[pattern])
    }

    pub fn animate(&mut self, on: bool) -> anyhow::Result<()> {
        self.send(CMD_ANIMATE, &[on as u8])
    }

    pub fn set_color(&mut self, r: u8, g: u8, b: u8) -> anyhow::Result<()> {
        self.send(CMD_SET_COLOR, &[r, g, b])
    }

    pub fn sleep(&mut self, on: bool) -> anyhow::Result<()> {
        self.send(CMD_SLEEP, &[on as u8])
    }
}

