use std::io::Write;
use std::{io::Read, net::TcpStream, time::Duration};

use crossterm::event::{poll, read, Event, KeyCode, KeyModifiers};
use postcard::experimental::max_size::MaxSize;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

#[derive(Default)]
pub struct App {
    find_state: crate::find::FindState,
}

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> anyhow::Result<()> {
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
                            return Ok(());
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
        .constraints([Constraint::Length(2), Constraint::Min(1)].as_ref())
        .split(f.size());

    let title = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::new().dark_gray())
        .title(format!("Controlling {}", app.find_state.ip.unwrap()))
        .title_alignment(Alignment::Center)
        .title_style(Style::new().reset());

    let paragraph = Paragraph::new(vec![Line::from("Your implementation here!")])
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::new().dark_gray()),
        );

    f.render_widget(title, chunks[0]);
    f.render_widget(paragraph, chunks[1]);
}
