// src/presets.rs
use crate::device::LedDevice;

pub enum Preset {
    Rainbow,
    Solid(u8, u8, u8),
    FullBright,
}

pub fn apply(dev: &mut LedDevice, preset: Preset) -> anyhow::Result<()> {
    match preset {
        Preset::Rainbow => {
            dev.set_pattern(0x01)?;
            dev.animate(true)?;
        }
        Preset::Solid(r, g, b) => dev.set_color(r, g, b)?,
        Preset::FullBright => dev.set_pattern(0x05)?,
    }
    Ok(())
}

