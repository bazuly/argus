use crate::models::{DevProcess, DockerContainer, PortBinding};
use crate::output::table::format_port_owner;
use crate::tui::app::{App, Tab};

pub fn find_matches(app: &App) -> Vec<usize> {
    let Some(snapshot) = &app.snapshot else {
        return Vec::new();
    };

    let query = app.search_query.trim();
    if query.is_empty() {
        return Vec::new();
    }
    let query = query.to_lowercase();

    match app.tab {
        Tab::Ports => snapshot
            .ports
            .iter()
            .enumerate()
            .filter(|(_, binding)| port_matches(binding, &query))
            .map(|(index, _)| index)
            .collect(),
        Tab::Processes => snapshot
            .processes
            .iter()
            .enumerate()
            .filter(|(_, process)| process_matches(process, &query))
            .map(|(index, _)| index)
            .collect(),
        Tab::Docker => snapshot
            .containers
            .iter()
            .enumerate()
            .filter(|(_, container)| container_matches(container, &query))
            .map(|(index, _)| index)
            .collect(),
    }
}

fn port_matches(binding: &PortBinding, query: &str) -> bool {
    binding.port.to_string().contains(query)
        || binding.address.to_ascii_lowercase().contains(query)
        || format_port_owner(binding).contains(query)
        || binding
            .pid
            .map(|pid| pid.to_string().contains(query))
            .unwrap_or(false)
}

fn process_matches(process: &DevProcess, query: &str) -> bool {
    process.pid.to_string().contains(query)
        || process.name.to_ascii_lowercase().contains(query)
        || process.cmdline.to_ascii_lowercase().contains(query)
}

fn container_matches(container: &DockerContainer, query: &str) -> bool {
    container.name.to_ascii_lowercase().contains(query)
        || container.image.to_ascii_lowercase().contains(query)
        || container.status.to_ascii_lowercase().contains(query)
        || format_ports(container).contains(query)
}

fn format_ports(container: &DockerContainer) -> String {
    if container.host_ports.is_empty() {
        return String::new();
    }

    container
        .host_ports
        .iter()
        .map(|port| port.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
