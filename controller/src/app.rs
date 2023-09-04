use std::time::Duration;

use crossterm::event::{poll, read, Event, KeyCode, KeyModifiers};
use ratatui::{prelude::*, widgets::{Block, Borders}};

#[derive(Default)]
pub struct App {
    find_state: crate::find::FindState
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> std::io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if poll(Duration::from_millis(500))? {
            if let Event::Key(key) = read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    KeyCode::Char('c') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            return Ok(())
                        }
                    }
                    _ => {}
                }

            }
        }
        app.find_state.update();
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    if app.find_state.ip.is_none() {
        return app.find_state.ui(f);
    }
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::new().dark_gray())
        .title(format!("Controlling {}", app.find_state.ip.unwrap()))
        .title_alignment(Alignment::Center)
        .title_style(Style::new().reset());

    f.render_widget(title, chunks[0]);
}
