use crate::model::SystemStats;
use anyhow::Result;
use std::thread;
use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, System};

pub fn collect() -> Result<SystemStats> {
    let mut system = System::new();
    system.refresh_memory();
    system.refresh_cpu_usage();
    thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
    system.refresh_cpu_usage();

    let stats = SystemStats {
        total_memory: system.total_memory(),
        used_memory: system.used_memory(),
        global_cpu_usage: system.global_cpu_usage(),
    };

    Ok(stats)
}
