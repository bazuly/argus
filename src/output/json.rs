use crate::models::{DevProcess, PortBinding, SystemStats};
use anyhow::Result;
use serde_json::to_string_pretty;

pub fn print_ports(bindings: &[PortBinding]) -> Result<()> {
    let json_string = to_string_pretty(bindings)?;

    println!("{json_string}");
    Ok(())
}

pub fn print_processes(processes: &[DevProcess]) -> Result<()> {
    let json_string = serde_json::to_string_pretty(processes)?;
    println!("{json_string}");
    Ok(())
}

// not massive, cuz SystemStats mono structure
pub fn print_stats(stats: &SystemStats) -> Result<()> {
    let json_string = serde_json::to_string_pretty(stats)?;
    println!("{json_string}");
    Ok(())
}
