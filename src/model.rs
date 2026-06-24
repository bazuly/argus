use crate::collectors::models::Protocol;

pub struct PortBinding {
    pub port: u16,
    pub protocol: Protocol,
    pub address: String,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
}

pub struct DevProcess {
    pub pid: u32,
    pub name: String,
    pub cmdline: String,
    pub memory_bytes: u64,
    pub cpu_usage: f32,
    pub is_dev: bool,
}

pub struct SystemStats {
    pub total_memory: u64,
    pub used_memory: u64,
    pub global_cpu_usage: f32,
}
