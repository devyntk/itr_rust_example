use std::io::Write;
use std::{io::Read, net::TcpStream, time::Duration};

use crossterm::event::{poll, read, Event, KeyCode, KeyModifiers};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

#[derive(Default)]
pub struct App {
    find_state: crate::find::FindState,
    sock: Option<TcpStream>,
    light: bool,
    temp: f32,
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
                    KeyCode::Char(' ') => {
                        app.light = !app.light;
                    }
                    _ => {}
                }
            }
        }
        app.find_state.update();
        if let Some(ip) = app.find_state.ip {
            if app.sock.is_none() {
                app.sock = Some(TcpStream::connect((ip, shared::APPLICATION_PORT))?);
            }

            let mut sock = app.sock.take().unwrap();

            let controller_msg = shared::ControllerMsg {
                light_on: app.light,
            };
            sock.write(&postcard::to_stdvec(&controller_msg)?)?;

            let mut read_vec = vec![];
            sock.read(&mut read_vec)?;
            let device_msg: shared::DeviceMsg = postcard::from_bytes(&read_vec)?;
            app.temp = device_msg.internal_temp;
        }
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

    let paragraph = Paragraph::new(vec![
        Line::from(format!(
            "Light on: {:?} (press spacebar to change)",
            app.light
        )),
        Line::from(format!("Internal temp: {:?}", app.temp)),
    ])
    .style(Style::default().fg(Color::Gray))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::new().dark_gray()),
    );

    f.render_widget(title, chunks[0]);
    f.render_widget(paragraph, chunks[1]);
}
