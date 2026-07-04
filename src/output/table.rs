use crate::models::{DevProcess, PortBinding, Protocol, SystemStats};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ContentArrangement, Table};

const BYTES_IN_MB: f64 = 1024.0 * 1024.0;
const BYTES_IN_GB: f64 = 1024.0 * 1024.0 * 1024.0;
const MAX_CMDLINE_LEN: usize = 60;

pub fn print_ports(bindings: &[PortBinding]) {
    if bindings.is_empty() {
        println!("No listening ports found");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(["PORT", "PROTO", "ADDRESS", "PID", "OWNER"]);

    for binding in bindings {
        table.add_row([
            binding.port.to_string(),
            format_protocol(binding.protocol),
            binding.address.clone(),
            format_pid(binding.pid),
            format_port_owner(binding),
        ]);
    }
    println!("{table}");
}

pub fn print_processes(processes: &[DevProcess]) {
    if processes.is_empty() {
        println!("processes not found");
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(["PID", "DEV", "CPU", "MEM", "NAME", "CMD"]);

    for process in processes {
        table.add_row([
            process.pid.to_string(),
            format_is_dev(process.is_dev),
            format_cpu(process.cpu_usage),
            format_memory_mb(process.memory_bytes),
            process.name.clone(),
            truncate_text(&process.cmdline, MAX_CMDLINE_LEN),
        ]);
    }
    println!("{table}")
}

pub fn print_stats(stats: &SystemStats) {
    let used_gb = format_memory_gb(stats.used_memory);
    let total_gb = format_memory_gb(stats.total_memory);

    let used_percent = if stats.total_memory > 0 {
        stats.used_memory as f64 / stats.total_memory as f64 * 100.0
    } else {
        0.0
    };

    println!("RAM:  {used_gb} / {total_gb}  ({used_percent:.1}% used)");
    println!("CPU:  {}", format_cpu(stats.global_cpu_usage));
}

fn format_memory_mb(bytes: u64) -> String {
    let megabytes = bytes as f64 / BYTES_IN_MB;
    format!("{megabytes:.1} MB")
}

fn format_memory_gb(bytes: u64) -> String {
    let gigabytes = bytes as f64 / BYTES_IN_GB;
    format!("{gigabytes:.1} GB")
}

fn format_protocol(protocol: Protocol) -> String {
    match protocol {
        Protocol::Tcp => "TCP".to_string(),
        Protocol::Udp => "UDP".to_string(),
    }
}

fn format_pid(pid: Option<u32>) -> String {
    match pid {
        Some(value) => value.to_string(),
        None => "-".to_string(),
    }
}

fn format_cpu(usage: f32) -> String {
    format!("{usage:.1}%")
}

fn format_is_dev(is_dev: bool) -> String {
    if is_dev {
        "yes".to_string()
    } else {
        "no".to_string()
    }
}

pub fn truncate_text(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        return text.to_string();
    } else {
        let shortened: String = text.chars().take(max_len).collect();
        format!("{shortened}...")
    }
}

pub fn format_port_owner(binding: &PortBinding) -> String {
    if let Some(name) = &binding.container_name {
        return format!("{name} (docker)");
    }

    if let Some(process) = &binding.process_name {
        return process.clone();
    }

    "-".to_string()
}
