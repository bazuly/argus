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
}

#[derive(Serialize)]
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
