use std::{io::stdout, path::PathBuf};

use crate::gui::error::GrapeGuiError;

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        *,
    },
    Terminal,
};

pub fn device_selector(
    mut available_ports: Vec<PathBuf>,
) -> Result<Option<PathBuf>, GrapeGuiError> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut cursor = 0;
    let mut list_state = ListState::default().with_selected(Some(cursor));
    let n_ports = available_ports.len();
    let mut selected_port = None;
    loop {
        let title = Title::from(" Device Selector ".magenta().bold());
        let instructions = Title::from(Line::from(vec![
            " Navigate ".into(),
            "<Up>/<Down>".magenta().bold(),
            " Select ".into(),
            "<Enter>".magenta().bold(),
            " Quit ".into(),
            "<Q> ".magenta().bold(),
        ]));
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL);
        let port_names = available_ports.iter().map(|p| p.to_string_lossy());
        let list = List::new(port_names)
            .style(Style::default().fg(Color::White))
            .highlight_symbol(">>")
            .highlight_style(Style::default().fg(Color::Magenta))
            .block(block);
        list_state.select(Some(cursor));
        terminal
            .draw(|frame| {
                let area = frame.size();
                frame.render_stateful_widget(list, area, &mut list_state);
            })
            .unwrap();
        if event::poll(std::time::Duration::from_millis(16)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Down => {
                            cursor = (cursor + 1) % n_ports;
                        }
                        KeyCode::Up => {
                            cursor = (cursor + n_ports - 1) % n_ports;
                        }
                        KeyCode::Enter => {
                            selected_port = Some(cursor);
                            break;
                        }
                        KeyCode::Char('q') => break,
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(selected_port.map(|i| available_ports.swap_remove(i)))
}
