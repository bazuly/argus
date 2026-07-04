use crate::collectors::{ports, processes, system};
use crate::models::{DevProcess, PortBinding, SystemStats};
use anyhow::Result;

const TUI_IGNORED_PORTS: &[u16] = &[53, 323];

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
    Processes,
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

    pub fn set_tab(&mut self, tab: Tab) {
        self.tab = tab;
        self.selected_row = 0;
    }

    pub fn reload_snapshot(&mut self) -> Result<()> {
        let mut ports = ports::collect(None)?;
        // ignore default ports in tui
        ports.retain(|binding| !TUI_IGNORED_PORTS.contains(&binding.port));
        let processes = processes::collect(true)?;
        let stats = system::collect()?;

        if let Ok(containers) = crate::collectors::docker::collect() {
            crate::collectors::enrich::attach_docker(&mut ports, &containers);
        }

        self.snapshot = Some(Snapshot {
            ports,
            processes,
            stats,
        });

        self.last_error = None;

        Ok(())
    }
}
