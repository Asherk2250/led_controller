use sysinfo::System;

pub struct Stats {
    sys: System,
}

impl Stats {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu();
        sys.refresh_memory();
        Self { sys }
    }

    pub fn cpu_usage(&mut self) -> u8 {
        self.sys.refresh_cpu();
        let usage = self.sys.global_cpu_info().cpu_usage();
        (usage.clamp(0.0, 100.0) as u8).min(255)
    }

    pub fn ram_usage(&mut self) -> u8 {
        self.sys.refresh_memory();
        let total_memory = self.sys.total_memory();
        let used_memory = self.sys.used_memory();
        let usage = if total_memory > 0 {
            (used_memory as f32 / total_memory as f32) * 100.0
        } else {
            0.0
        };
        (usage.clamp(0.0, 100.0) as u8).min(255)
    }
}
