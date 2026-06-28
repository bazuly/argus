use crate::tui::app::App;
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // TODO: can quit or restart
    // with any layout
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

        _ => {}
    }
}
