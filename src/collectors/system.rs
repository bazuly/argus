use crate::models::SystemStats;
use anyhow::Result;
use std::thread;
use sysinfo::{Components, MINIMUM_CPU_UPDATE_INTERVAL, System};

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
        cpu_temp_c: read_cpu_temp_c(),
        gpu_temp_c: None,
    };

    Ok(stats)
}

fn read_cpu_temp_c() -> Option<f32> {
    let components = Components::new_with_refreshed_list();
    if components.is_empty() {
        return None;
    }

    let mut package: Option<f32> = None;
    let mut cores: Vec<f32> = Vec::new();
    let mut fallback: Vec<f32> = Vec::new();

    for component in &components {
        let Some(temp) = component.temperature() else {
            continue;
        };

        if !temp.is_finite() || temp <= 0.0 || temp > 250.0 {
            continue;
        }

        let label = component.label().to_ascii_lowercase();

        if is_non_cpu_sensor(&label) {
            continue;
        }
        if !looks_like_cpu_sensor(&label) {
            continue;
        }

        if is_package_sensor(&label) {
            package = Some(match package {
                Some(current_temp) => current_temp.max(temp),
                None => temp,
            });
        } else if is_core_sensor(&label) {
            cores.push(temp);
        } else {
            fallback.push(temp)
        }
    }

    // if package stats => return temp as package stats
    if let Some(temp) = package {
        return Some(temp);
    }

    // els return highest temp core, if no core stats, return other info (fallback) else None
    max_of(&cores).or_else(|| max_of(&fallback))
}

fn looks_like_cpu_sensor(label: &str) -> bool {
    const MARKERS: &[&str] = &[
        // Intel / x86
        "coretemp",
        "package id",
        "physical id",
        "x86_pkg_temp",
        "cpu package",
        "cpu die",
        "cpu core",
        "core ",
        // AMD
        "k10temp",
        "zenpower",
        "tctl",
        "tdie",
        "tccd",
        // ARM / SoC
        "cpu-thermal",
        "cpu_thermal",
        "soc-thermal",
        "soc_thermal",
        "cpu0-thermal",
        "cpu1-thermal",
        "cluster",
        "cpu-big",
        "cpu-little",
        "big-cpu",
        "little-cpu",
        // ACPI / generic
        "acpitz",
        "cpu",
    ];
    MARKERS.iter().any(|marker| label.contains(marker))
}

fn is_non_cpu_sensor(label: &str) -> bool {
    const NON_CPU_MARKERS: &[&str] = &[
        "gpu", "amdgpu", "nouveau", "nvidia", "nvme", "ssd", "hdd", "wifi", "wi-fi", "pch",
        "ambient", "battery",
    ];

    NON_CPU_MARKERS.iter().any(|marker| label.contains(marker))
}

fn is_package_sensor(label: &str) -> bool {
    label.contains("package")
        || label.contains("physical id")
        || label.contains("tctl")
        || label.contains("tdie")
        || label.contains("cpu-thermal")
        || label.contains("soc-thermal")
        || label == "soc"
}

fn is_core_sensor(label: &str) -> bool {
    label.contains("core ") || label.contains("coretemp")
}

fn max_of(values: &[f32]) -> Option<f32> {
    values
        .iter()
        .copied()
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
}
