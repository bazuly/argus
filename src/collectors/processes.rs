use crate::models::DevProcess;
use anyhow::Result;
use std::ffi::OsString;
use std::thread;
use sysinfo::{MINIMUM_CPU_UPDATE_INTERVAL, ProcessRefreshKind, ProcessesToUpdate, System};

pub fn collect(dev_only: bool) -> Result<Vec<DevProcess>> {
    let mut system = System::new();

    system.refresh_all();

    thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_cpu(),
    );

    let mut processes: Vec<DevProcess> = Vec::new();

    for process in system.processes().values() {
        let name: String = process.name().to_string_lossy().into_owned();

        let mut cmdline: String = format_cmdline(process.cmd());

        if cmdline.is_empty() {
            cmdline = name.clone();
        }

        let is_dev: bool = is_dev_process(&name, &cmdline);

        if dev_only && !is_dev {
            continue;
        }

        let pid: u32 = process.pid().as_u32();
        let memory_bytes: u64 = process.memory();
        let cpu_usage: f32 = process.cpu_usage();

        processes.push(DevProcess {
            pid,
            name,
            cmdline,
            memory_bytes,
            cpu_usage,
            is_dev,
        });
    }
    processes.sort_by(|left, right| right.memory_bytes.cmp(&left.memory_bytes));
    Ok(processes)
}

///   ["node", "/path/vite"] → "node /path/vite"
fn format_cmdline(cmd_parts: &[OsString]) -> String {
    let cmd_pieces: Vec<String> = cmd_parts
        .iter()
        .map(|part| part.to_string_lossy().into_owned())
        .collect();

    cmd_pieces.join(" ")
}

fn is_dev_process(name: &str, cmdline: &str) -> bool {
    const DEV_MARKERS: &[&str] = &[
        // JS
        "node",
        "npm",
        "pnpm",
        "yarn",
        "bun",
        "deno",
        "vite",
        "next",
        "nuxt",
        "nest",
        "webpack",
        "esbuild",
        "turbo",
        // Python
        "python",
        "uvicorn",
        "gunicorn",
        "django",
        "flask",
        "fastapi",
        "poetry",
        "celery",
        "manage.py",
        // Rust
        "cargo",
        "rustc",
        "target/debug",
        "target/release",
        // Go
        "go run",
        "air",
        // JVM
        "java",
        "gradle",
        "mvn",
        "maven",
        "spring",
        // .NET
        "dotnet",
        // Ruby / PHP
        "ruby",
        "rails",
        "puma",
        "php",
        "composer",
        "artisan",
        // DB / cache
        "postgres",
        "redis",
        "mongod",
        "mysql",
        "mariadb",
        "elasticsearch",
        "rabbitmq",
        "kafka",
        "minio",
        "memcached",
        // containers / k8s local
        "docker",
        "compose",
        "podman",
        "kubectl",
        "minikube",
        "kind",
        // local reverse proxy
        "nginx",
        "caddy",
        "traefik",
    ];

    let haystack: String = format!("{} {}", name, cmdline).to_lowercase();
    DEV_MARKERS.iter().any(|marker| haystack.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn detects_node_and_cargo_as_dev() {
        assert!(is_dev_process("node", "node server.js"));
        assert!(is_dev_process("cargo", "cargo run"));
        assert!(is_dev_process("python3", "uvicorn app:main"));
    }
    #[test]
    fn rejects_plain_shell() {
        assert!(!is_dev_process("bash", "-bash"));
        assert!(!is_dev_process("sleep", "sleep 10"));
    }
    #[test]
    fn format_cmdline_joins_parts() {
        let parts = [OsString::from("node"), OsString::from("/app/vite")];
        assert_eq!(format_cmdline(&parts), "node /app/vite");
    }
    #[test]
    fn format_cmdline_empty() {
        assert_eq!(format_cmdline(&[]), "");
    }
}
