use crate::models::DockerContainer;
use anyhow::{Context, Result};
use bollard::Docker;
use bollard::models::ContainerSummary;
use bollard::plugin::{ContainerStatsResponse, ContainerSummaryStateEnum};
use bollard::query_parameters::{ListContainersOptionsBuilder, StatsOptionsBuilder};
use futures_util::StreamExt;
use futures_util::future::join_all;
use tokio::runtime::Runtime;

pub fn collect() -> Result<Vec<DockerContainer>> {
    let runtime = Runtime::new().context("failed to create tokio runtime")?;

    runtime.block_on(collect_async())
}

async fn collect_async() -> Result<Vec<DockerContainer>> {
    let docker = Docker::connect_with_socket_defaults()
        .context("failed to connect to docker socket (/var/run/docker.sock)")?;

    // collect all docker containers, analogy "docker ps -a"
    let options = ListContainersOptionsBuilder::default().all(true).build();
    let summaries = docker.list_containers(Some(options)).await?;

    let mut containers: Vec<DockerContainer> = summaries
        .iter()
        .map(|summary| DockerContainer {
            id: container_id(summary),
            name: container_name(summary),
            image: short_image(summary.image.as_deref().unwrap_or("unknown")),
            status: container_status(summary.state.as_ref()),
            host_ports: extract_host_ports(summary),
            cpu_percent: None,
            memory_bytes: None,
        })
        .collect();

    let stats_jobs = containers
        .iter()
        .enumerate()
        .filter_map(|(index, container)| {
            if container.status != "running" {
                return None;
            }

            let docker = docker.clone();
            let container_id = container.id.clone();

            Some(async move {
                let stats = fetch_stats(&docker, &container_id).await;
                // return
                (index, stats)
            })
        });

    for (index, stats) in join_all(stats_jobs).await {
        if let Some((cpu, mem)) = stats {
            containers[index].cpu_percent = Some(cpu);
            containers[index].memory_bytes = Some(mem);
        }
    }
    containers.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(containers)
}

async fn fetch_stats(docker: &Docker, container_id: &str) -> Option<(f32, u64)> {
    let options = StatsOptionsBuilder::default().stream(false).build();
    // docker data stream
    let mut stream = docker.stats(container_id, Some(options));

    let response = stream.next().await?.ok()?;
    let cpu = calc_cpu_percent(&response)?;
    let mem = response.memory_stats.as_ref()?.usage?;

    Some((cpu, mem))
}

fn calc_cpu_percent(stats: &ContainerStatsResponse) -> Option<f32> {
    let cpu = stats.cpu_stats.as_ref()?;
    let precpu = stats.precpu_stats.as_ref()?;

    let cpu_total = cpu.cpu_usage.as_ref()?.total_usage?;
    let precpu_total = precpu.cpu_usage.as_ref()?.total_usage?;

    let system = cpu.system_cpu_usage?;
    let presystem = precpu.system_cpu_usage?;

    let cpu_delta = cpu_total as f64 - precpu_total as f64;
    let system_delta = system as f64 - presystem as f64;

    if cpu_delta <= 0.0 || system_delta <= 0.0 {
        return Some(0.0);
    }

    let online_cpus = cpu
        .online_cpus
        .map(|n| n as f64)
        .or_else(|| {
            cpu.cpu_usage
                .as_ref()?
                .percpu_usage
                .as_ref()
                .map(|cores| cores.len() as f64)
        })
        .unwrap_or(1.0);
    Some(((cpu_delta / system_delta) * online_cpus * 100.0) as f32)
}

fn extract_host_ports(summary: &ContainerSummary) -> Vec<u16> {
    let mut ports: Vec<u16> = Vec::new();

    let Some(port_list) = &summary.ports else {
        return ports;
    };

    for port in port_list {
        if let Some(public_port) = port.public_port {
            ports.push(public_port);
        }
    }

    ports
}

fn short_image(image: &str) -> String {
    image.rsplit('/').next().unwrap_or(image).to_string()
}

fn container_id(summary: &ContainerSummary) -> String {
    summary
        .id
        .clone()
        // edge case, better catch container_name instead of None
        .unwrap_or_else(|| container_name(summary))
}

fn container_name(summary: &ContainerSummary) -> String {
    summary
        .names
        .as_ref()
        .and_then(|names| names.first())
        .map(|name| name.trim_start_matches("/").to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn container_status(state: Option<&ContainerSummaryStateEnum>) -> String {
    use ContainerSummaryStateEnum::*;
    let label = match state {
        Some(RUNNING) => "running",
        Some(EXITED) => "exited",
        Some(CREATED) => "created",
        Some(PAUSED) => "paused",
        Some(RESTARTING) => "restarting",
        Some(REMOVING) => "removing",
        Some(DEAD) => "dead",
        Some(EMPTY) | None => "unknown",
        Some(STOPPING) => "stopping",
    };

    label.to_string()
}
