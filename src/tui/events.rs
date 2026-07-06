use crate::tui::app::{App, InputMode, Tab};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    if app.input_mode == InputMode::Search {
        handle_search_key(app, key);
        return;
    }

    handle_normal_key(app, key);
}

fn handle_search_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.cancel_search();
        }

        KeyCode::Enter => {
            app.apply_search(0);
            app.input_mode = InputMode::Normal;
        }

        KeyCode::Backspace => {
            app.pop_search_char();
        }

        KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.push_search_char(ch);
        }

        _ => {}
    }
}

fn handle_normal_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.should_quit = true;
        }

        KeyCode::Esc => {
            app.should_quit = true;
        }

        KeyCode::Char('/') => {
            app.start_search();
        }

        KeyCode::Char('n') => {
            app.apply_search(1);
        }

        KeyCode::Char('N') => {
            app.apply_search(-1);
        }

        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.needs_refresh = true;
        }

        KeyCode::Char('1') => app.set_tab(Tab::Ports),
        KeyCode::Char('2') => app.set_tab(Tab::Processes),
        KeyCode::Char('3') => app.set_tab(Tab::Docker),

        KeyCode::Tab => {
            let next_tab = match app.tab {
                Tab::Ports => Tab::Processes,
                Tab::Processes => Tab::Docker,
                Tab::Docker => Tab::Ports,
            };
            app.set_tab(next_tab);
        }

        KeyCode::Up | KeyCode::Char('k') => app.move_selection(-1),
        KeyCode::Down | KeyCode::Char('j') => app.move_selection(1),
        KeyCode::PageUp => app.move_selection(-20),
        KeyCode::PageDown => app.move_selection(20),

        KeyCode::Home => {
            app.selected_row = 0;
            app.table_state.select(Some(0));
        }

        KeyCode::End => {
            let last = app.active_list_len().saturating_sub(1);
            app.selected_row = last;
            app.table_state.select(Some(last));
        }

        KeyCode::Char('x') | KeyCode::Char('X') => {
            if app.tab == Tab::Processes {
                app.kill_selected_process();
            }
        }

        _ => {}
    }
}
