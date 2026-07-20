use crate::tui::app::App;
use crate::tui::{events, ui};
use anyhow::Result;
use crossterm::event::{self, Event};
use std::time::{Duration, Instant};

pub fn run() -> Result<()> {
    let result: Result<()> = ratatui::run(|terminal| {
        let mut app = App::new();

        let refresh_every = Duration::from_secs(5);
        let mut last_refresh = Instant::now() - refresh_every;

        loop {
            if app.should_quit {
                break;
            }

            let time_to_refresh = last_refresh.elapsed() >= refresh_every;
            if app.needs_refresh || time_to_refresh {
                match app.reload_snapshot() {
                    Err(error) => {
                        app.last_error = Some(error.to_string());
                    }
                    Ok(_) => {}
                }
                app.needs_refresh = false;
                last_refresh = Instant::now();
            }

            terminal.draw(|frame| ui::draw(frame, &mut app))?;

            // wait key for block
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key_event) = event::read()? {
                    events::handle_key(&mut app, key_event);
                }
            }
        }

        Ok(())
    });
    result
}
