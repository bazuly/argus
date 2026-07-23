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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Protocol;

    fn port(port: u16) -> PortBinding {
        PortBinding {
            port,
            protocol: Protocol::Tcp,
            address: "127.0.0.1".to_string(),
            pid: None,
            process_name: None,
            container_name: None,
            container_image: None,
        }
    }

    fn container(name: &str, image: &str, host_ports: Vec<u16>) -> DockerContainer {
        DockerContainer {
            id: format!("id-{name}"),
            name: name.to_string(),
            image: image.to_string(),
            status: "running".to_string(),
            host_ports,
            cpu_percent: None,
            memory_bytes: None,
        }
    }

    #[test]
    fn attaches_name_and_image_when_port_matches() {
        let mut ports = vec![port(6379)];
        let containers = vec![container("redis-dev", "redis:7", vec![6379])];

        attach_docker(&mut ports, &containers);

        assert_eq!(ports[0].container_name.as_deref(), Some("redis-dev"));
        assert_eq!(ports[0].container_image.as_deref(), Some("redis:7"));
    }

    #[test]
    fn leaves_fields_empty_when_no_port_match() {
        let mut ports = vec![port(8080)];
        let containers = vec![container("redis-dev", "redis:7", vec![6379])];

        attach_docker(&mut ports, &containers);

        assert!(ports[0].container_name.is_none());
        assert!(ports[0].container_image.is_none());
    }

    #[test]
    fn uses_first_matching_container() {
        let mut ports = vec![port(5432)];
        let containers = vec![
            container("postgres-a", "postgres:16", vec![5432]),
            container("postgres-b", "postgres:15", vec![5432]),
        ];
        attach_docker(&mut ports, &containers);
        assert_eq!(ports[0].container_name.as_deref(), Some("postgres-a"));
        assert_eq!(ports[0].container_image.as_deref(), Some("postgres:16"));
    }

    #[test]
    fn only_matching_ports_in_list() {
        let mut ports = vec![port(6379), port(8080)];
        let containers = vec![container("redis-dev", "redis:7", vec![6379])];

        attach_docker(&mut ports, &containers);

        assert_eq!(ports[0].container_name.as_deref(), Some("redis-dev"));
        assert!(ports[1].container_name.is_none());
    }
}
