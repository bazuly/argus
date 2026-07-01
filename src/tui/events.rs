use crate::tui::app::{App, Tab};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // TODO: can quit or restart
    // with any keyboard layout
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.should_quit = true;
        }

        KeyCode::Esc => {
            app.should_quit = true;
        }

        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.needs_refresh = true;
        }

        KeyCode::Char('1') => {
            app.set_tab(Tab::Ports);
        }

        KeyCode::Char('2') => {
            app.set_tab(Tab::Processes);
        }

        KeyCode::Tab => {
            let next_tab = match app.tab {
                Tab::Ports => Tab::Processes,
                Tab::Processes => Tab::Ports,
            };
            app.set_tab(next_tab);
        }

        _ => {}
    }
}
