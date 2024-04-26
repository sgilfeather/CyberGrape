use std::{fmt::Display, io::stdout, path::PathBuf, sync::mpsc, thread::spawn};

use crate::gui::error::GrapeGuiError;

use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use nom::character;
use ratatui::{
    prelude::*,
    widgets::{
        block::{Position, Title},
        *,
    },
    Terminal,
};


enum ThreadMessage {
    Stop
}

pub fn fold_until_stop<F, T, E>(init: T, f: F) -> Result<T, GrapeGuiError>
where
    F: Fn(T) -> T + Send + Sync + 'static,
    T: Send + Sync + 'static
{
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let (stop_tx, stop_rx) = mpsc::channel();
    let (res_tx, res_rx) = mpsc::channel();

    let th = spawn(move || {
        let mut val = init;

        loop {
            val = f(val);
            if let Ok(ThreadMessage::Stop) = stop_rx.try_recv() {
                res_tx.send(val).unwrap();
                break
            }
        }

    });


    loop {
        let title = Title::from(" Device Selector ".magenta().bold());
        let instructions = Title::from(Line::from(vec![
            " Things are happening! ".into(),
            " Press the spacebar to stop ".into(),
        ]));
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL);
        terminal
            .draw(|frame| {
                let area = frame.size();
                frame.render_widget(block, area);
            })
            .unwrap();
        if event::poll(std::time::Duration::from_millis(16)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    break;
                }
            }
        }
    }

    stop_tx.send(ThreadMessage::Stop)?;
    let res = res_rx.recv()?;
    th.join().map_err(|_| GrapeGuiError::JoinError)?;
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;    

    Ok(res)
}
