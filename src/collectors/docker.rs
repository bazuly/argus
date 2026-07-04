use crate::models::DockerContainer;
use anyhow::{Context, Result};
use bollard::Docker;
use bollard::models::ContainerSummary;
use bollard::plugin::ContainerSummaryStateEnum;
use bollard::query_parameters::ListContainersOptionsBuilder;
use tokio::runtime::Runtime;

pub fn collect() -> Result<Vec<DockerContainer>> {
    let runtime = Runtime::new().context("failed to create tokio runtime")?;

    runtime.block_on(collect_async())
}

async fn collect_async() -> Result<Vec<DockerContainer>> {
    let docker = Docker::connect_with_socket_defaults()
        .context("failed to connect to docker socket (/var/run/docker.sock)")?;

    let options = ListContainersOptionsBuilder::default().build();
    let summaries = docker.list_containers(Some(options)).await?;

    let mut result: Vec<DockerContainer> = Vec::new();

    for summary in summaries {
        let host_ports = extract_host_ports(&summary);

        result.push(DockerContainer {
            name: container_name(&summary),
            image: short_image(summary.image.as_deref().unwrap_or("unknown")),
            status: container_status(summary.state.as_ref()),
            host_ports,
        });
    }
    Ok(result)
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
