use crate::collectors::{ports, processes, system};
use crate::models::{DevProcess, PortBinding, SystemStats};
use anyhow::Result;

pub struct Snapshot {
    pub ports: Vec<PortBinding>,
    pub processes: Vec<DevProcess>,
    pub stats: SystemStats,
}

// TUI app state
pub struct App {
    pub snapshot: Option<Snapshot>, // None before first refresh
    pub tab: Tab,
    pub selected_row: usize,
    pub should_quit: bool,
    pub needs_refresh: bool,
    pub last_error: Option<String>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Ports,
    // Processes,
    // Stats,
}

impl App {
    pub fn new() -> Self {
        Self {
            snapshot: None,
            tab: Tab::Ports,
            selected_row: 0,
            should_quit: false,
            needs_refresh: true,
            last_error: None,
        }
    }

    pub fn reload_snapshot(&mut self) -> Result<()> {
        let ports = ports::collect(None)?;
        let processes = processes::collect(true)?;
        let stats = system::collect()?;

        self.snapshot = Some(Snapshot {
            ports,
            processes,
            stats,
        });

        self.last_error = None;

        Ok(())
    }
}
