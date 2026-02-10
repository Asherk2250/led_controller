// src/commands.rs

pub const MAGIC1: u8 = 0x32;
pub const MAGIC2: u8 = 0xAC;

pub const CMD_BRIGHTNESS: u8 = 0x00;
pub const CMD_PATTERN: u8 = 0x01;
pub const CMD_SLEEP: u8 = 0x03;
pub const CMD_ANIMATE: u8 = 0x04;
pub const CMD_SET_COLOR: u8 = 0x13;
pub const CMD_START_GAME: u8 = 0x10;
pub const CMD_VERSION: u8 = 0x20;

pub fn brightness(level: u8) -> Vec<u8> {
    vec![MAGIC1, MAGIC2, 0x00, level]
}

pub fn percentage(percent: u8) -> Vec<u8> {
    vec![MAGIC1, MAGIC2, 0x01, 0x00, percent]
}
