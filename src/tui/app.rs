use crate::collectors::docker::collect;
use crate::collectors::{enrich, ports, processes, system};
use crate::models::{DevProcess, DockerContainer, PortBinding, SystemStats};
use crate::tui::search;
use anyhow::Result;

const TUI_IGNORED_PORTS: &[u16] = &[53, 323];

pub struct Snapshot {
    pub ports: Vec<PortBinding>,
    pub processes: Vec<DevProcess>,
    pub containers: Vec<DockerContainer>,
    pub docker_error: Option<String>,
    pub stats: SystemStats,
}

// TUI app state
pub struct App {
    pub snapshot: Option<Snapshot>, // None before first refresh
    pub tab: Tab,
    pub selected_row: usize,
    pub list_offset: usize, // first visible row without header
    pub table_state: ratatui::widgets::TableState, // ratatui default table state
    pub should_quit: bool,
    pub needs_refresh: bool,
    pub last_error: Option<String>,
    pub input_mode: InputMode,
    pub search_query: String,
    pub search_match_index: usize,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Ports,
    Processes,
    Docker,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputMode {
    Normal,
    Search,
}

impl App {
    pub fn new() -> Self {
        Self {
            snapshot: None,
            tab: Tab::Ports,
            selected_row: 0,
            list_offset: 0,
            table_state: ratatui::widgets::TableState::default(),
            should_quit: false,
            needs_refresh: true,
            last_error: None,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            search_match_index: 0,
        }
    }

    pub fn set_tab(&mut self, tab: Tab) {
        self.tab = tab;
        self.selected_row = 0;
        self.list_offset = 0;
        self.search_query.clear();
        self.input_mode = InputMode::Normal;
        self.search_match_index = 0;
    }

    pub fn reload_snapshot(&mut self) -> Result<()> {
        let mut ports = ports::collect(None)?;
        // ignore default ports in tui
        ports.retain(|binding| !TUI_IGNORED_PORTS.contains(&binding.port));
        let processes = processes::collect(true)?;
        let stats = system::collect()?;

        let (containers, docker_error) = match collect() {
            Ok(containers) => {
                enrich::attach_docker(&mut ports, &containers);
                (containers, None)
            }
            Err(error) => (Vec::new(), Some(error.to_string())),
        };

        self.snapshot = Some(Snapshot {
            ports,
            processes,
            containers,
            docker_error,
            stats,
        });

        self.last_error = None;

        self.clamp_selection_after_refresh();

        Ok(())
    }

    pub fn active_list_len(&self) -> usize {
        let Some(snapshot) = &self.snapshot else {
            return 0;
        };

        match self.tab {
            Tab::Ports => snapshot.ports.len(),
            Tab::Processes => snapshot.processes.len(),
            Tab::Docker => snapshot.containers.len(),
        }
    }

    pub fn move_selection(&mut self, delta: isize) {
        let len = self.active_list_len();
        if len == 0 {
            return;
        }

        let max = len - 1;
        let next = (self.selected_row as isize + delta).clamp(0, max as isize) as usize;
        self.selected_row = next;
        self.table_state.select(Some(next));
    }

    pub fn clamp_selection_after_refresh(&mut self) {
        let len = self.active_list_len();
        if len == 0 {
            self.selected_row = 0;
            self.list_offset = 0;
            self.table_state.select(None);
            return;
        }
        if self.selected_row >= len {
            self.selected_row = len - 1;
        }
        self.table_state.select(Some(self.selected_row))
    }

    pub fn ensure_visible(&mut self, viewport_rows: usize) {
        if viewport_rows == 0 {
            return;
        }
        if self.selected_row < self.list_offset {
            self.list_offset = self.selected_row;
        } else if self.selected_row >= self.list_offset + viewport_rows {
            self.list_offset = self.selected_row - viewport_rows + 1;
        }
    }

    pub fn start_search(&mut self) {
        self.input_mode = InputMode::Search;
        self.search_query.clear();
        self.search_match_index = 0;
    }

    pub fn cancel_search(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_query.clear();
        self.search_match_index = 0;
    }

    pub fn push_search_char(&mut self, ch: char) {
        self.search_query.push(ch);
    }

    pub fn pop_search_char(&mut self) {
        self.search_query.pop();
    }

    pub fn apply_search(&mut self, step: isize) {
        let matches = search::find_matches(self);
        if matches.is_empty() {
            return;
        }

        let count = matches.len();
        let index = if step == 0 {
            0
        } else {
            (self.search_match_index as isize + step).rem_euclid(count as isize) as usize
        };

        self.search_match_index = index;
        let row = matches[index];
        self.selected_row = row;
        self.table_state.select(Some(row));
    }

    pub fn search_status(&self) -> Option<String> {
        if self.search_query.trim().is_empty() {
            return None;
        }
        let matches = search::find_matches(self);
        if matches.is_empty() {
            return Some(format!("/{}", self.search_query) + "  (no matches)");
        }
        Some(format!(
            "/{}  [{}/{}]",
            self.search_query,
            self.search_match_index + 1,
            matches.len()
        ))
    }
}
