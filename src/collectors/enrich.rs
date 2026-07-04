use crate::models::{DockerContainer, PortBinding};

pub fn attach_docker(ports: &mut [PortBinding], containers: &[DockerContainer]) {
    for binding in ports.iter_mut() {
        for container in containers {
            if container.host_ports.contains(&binding.port) {
                binding.container_name = Some(container.name.clone());
                binding.container_image = Some(container.image.clone());
                break;
            }
        }
    }
}
