use crate::models::Protocol;
use crate::tui::app::{App, Tab};

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Paragraph, Row, Table};

const BYTES_IN_GB: f64 = 1024.0 * 1024.0 * 1024.0;

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header: RAM/CPU
            Constraint::Length(1), // tabs
            Constraint::Min(0),    // main table
            Constraint::Length(1), // footer: hotkeys
        ])
        .split(frame.area());
    draw_header(frame, chunks[0], app);
    draw_tabs(frame, chunks[1], app);
    draw_main(frame, chunks[2], app);
    draw_footer(frame, chunks[3]);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let text = if let Some(error) = &app.last_error {
        format!("argus | ERROR: {error}")
    } else if let Some(snapshot) = &app.snapshot {
        let used_gb = bytes_to_gb(snapshot.stats.used_memory);
        let total_gb = bytes_to_gb(snapshot.stats.total_memory);
        let cpu = snapshot.stats.global_cpu_usage;
        format!("argus | RAM {used_gb:.1}/{total_gb:.1} GB | CPU {cpu:.1}%")
    } else {
        "argus | Loading...".to_string()
    };
    let widget = Paragraph::new(text).block(Block::bordered().title(" DevTop "));
    frame.render_widget(widget, area);
}

fn draw_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let label = match app.tab {
        Tab::Ports => "[Ports]",
    };
    let widget = Paragraph::new(label);
    frame.render_widget(widget, area);
}

fn draw_main(frame: &mut Frame, area: Rect, app: &App) {
    match app.tab {
        Tab::Ports => draw_ports_table(frame, area, app),
    }
}

fn draw_ports_table(frame: &mut Frame, area: Rect, app: &App) {
    let Some(snapshot) = &app.snapshot else {
        let widget = Paragraph::new("Loading ports...").block(Block::bordered().title("Ports"));
        frame.render_widget(widget, area);
        return;
    };
    if snapshot.ports.is_empty() {
        let widget =
            Paragraph::new("No listening ports found.").block(Block::bordered().title("Ports"));
        frame.render_widget(widget, area);
        return;
    }
    let header = Row::new(vec!["PORT", "PROTO", "ADDRESS", "PID", "PROCESS"])
        .style(Style::new().add_modifier(Modifier::BOLD))
        .bottom_margin(1);
    let rows: Vec<Row> = snapshot
        .ports
        .iter()
        .map(|binding| {
            Row::new(vec![
                binding.port.to_string(),
                format_protocol(binding.protocol),
                binding.address.clone(),
                format_pid(binding.pid),
                format_optional_text(&binding.process_name),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Min(10),
            Constraint::Length(8),
            Constraint::Min(10),
        ],
    )
    .header(header)
    .block(Block::bordered().title("Ports"));
    frame.render_widget(table, area);
}

fn draw_footer(frame: &mut Frame, area: Rect) {
    let text = "q: quit | r: refresh";
    let widget = Paragraph::new(text);
    frame.render_widget(widget, area);
}

// formatting only for representation
fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / BYTES_IN_GB
}

fn format_protocol(protocol: Protocol) -> String {
    match protocol {
        Protocol::Tcp => "tcp".to_string(),
        Protocol::Udp => "udp".to_string(),
    }
}

fn format_pid(pid: Option<u32>) -> String {
    match pid {
        Some(value) => value.to_string(),
        None => "-".to_string(),
    }
}

fn format_optional_text(value: &Option<String>) -> String {
    match value {
        Some(text) => text.clone(),
        None => "-".to_string(),
    }
}
