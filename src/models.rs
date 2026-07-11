use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
}
#[derive(Serialize)]
pub struct PortBinding {
    pub port: u16,
    pub protocol: Protocol,
    pub address: String,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub container_name: Option<String>,
    pub container_image: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct DevProcess {
    pub pid: u32,
    pub name: String,
    pub cmdline: String,
    pub memory_bytes: u64,
    pub cpu_usage: f32,
    pub is_dev: bool,
}

#[derive(Serialize)]
pub struct SystemStats {
    pub total_memory: u64,
    pub used_memory: u64,
    pub global_cpu_usage: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub host_ports: Vec<u16>,
    pub cpu_percent: Option<f32>,
    pub memory_bytes: Option<u64>,
}
