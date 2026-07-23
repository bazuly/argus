use crate::models::SystemStats;
use anyhow::Result;
use std::process::Command;
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
    read_cpu_temp_from_sysinfo().or_else(|| {
        if is_wsl() {
            read_cpu_temp_from_windows_acpi()
        } else {
            None
        }
    })
}

fn read_cpu_temp_from_sysinfo() -> Option<f32> {
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

fn is_wsl() -> bool {
    std::env::var_os("WSL_DISTRO_NAME").is_some()
        || std::fs::read_to_string("/proc/version")
            .map(|v| v.to_ascii_lowercase().contains("microsoft"))
            .unwrap_or(false)
}

fn max_of(values: &[f32]) -> Option<f32> {
    values
        .iter()
        .copied()
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
}

fn read_cpu_temp_from_windows_acpi() -> Option<f32> {
    // CurrentTemperature is in tenths of Kelvin.
    // Celsius = CurrentTemperature / 10 - 273.15
    // We print one number per zone; Rust takes the max.
    const PS: &str = r#"
$ErrorActionPreference = 'Stop'
$zones = Get-CimInstance -Namespace root/wmi -ClassName MSAcpi_ThermalZoneTemperature -ErrorAction Stop
foreach ($z in $zones) {
    $c = ($z.CurrentTemperature / 10.0) - 273.15
    if ($c -gt 0 -and $c -lt 150) {
        Write-Output $c
    }
}
"#;
    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            PS,
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_acpi_temps(&stdout)
}

fn parse_acpi_temps(stdout: &str) -> Option<f32> {
    let mut temps: Vec<f32> = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // PowerShell may print "61.05" or "61,05" depending on locale
        let normalized_temp = line.replace(',', ".");
        if let Ok(temp) = normalized_temp.parse::<f32>() {
            if temp.is_finite() && temp > 0.0 && temp < 150.0 {
                temps.push(temp);
            }
        }
    }
    // parse to float => push into temp vector => get max temp value
    max_of(&temps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_acpi_temps_takes_max() {
        assert_eq!(parse_acpi_temps("45.0\n61.5\n50\n"), Some(61.5));
    }

    #[test]
    fn parse_acpi_temps_accepts_comma_decimal() {
        assert_eq!(parse_acpi_temps("61,5\n"), Some(61.5));
    }

    #[test]
    fn parse_acpi_temps_empty() {
        assert_eq!(parse_acpi_temps(""), None);
        assert_eq!(parse_acpi_temps("nope\n"), None);
    }

    #[test]
    fn looks_like_cpu_and_not_gpu() {
        assert!(looks_like_cpu_sensor("package id 0"));
        assert!(looks_like_cpu_sensor("cpu-thermal"));
        assert!(is_non_cpu_sensor("amdgpu"));
        assert!(!is_non_cpu_sensor("coretemp"));
    }

    #[test]
    fn max_of_works() {
        assert_eq!(max_of(&[1.0, 3.0, 2.0]), Some(3.0));
        assert_eq!(max_of(&[]), None);
    }
}
