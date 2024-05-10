use std::{io::stdout, sync::mpsc, thread::spawn};

use crate::gui::error::GrapeGuiError;

use crossterm::{
    event::{self, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use ratatui::{
    prelude::*,
    widgets::{block::Title, *},
    Terminal,
};

enum ThreadMessage {
    Stop,
}

/// Generates a gui that runs a function until the user provides input.
///
/// The function can be thought of as a recursive fold. `init` contains the
/// inital state of the loop, then `f` is called on the inital state to produce
/// a new state, and then `f` is called on that new state, and so on until the
/// user indicates that this should stop.
pub fn fold_until_stop<F, T>(init: T, f: F) -> Result<T, GrapeGuiError>
where
    F: Fn(T) -> T + Send + Sync + 'static,
    T: Send + Sync + 'static,
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
                break;
            }
        }
    });

    loop {
        let title = Title::from(" Monitoring Tag Positions... ".magenta().bold());
        let text = Paragraph::new(Line::from(vec![
            " Things are happening! ".into(),
            " Press any key to stop ".into(),
        ]));
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .borders(Borders::ALL);
        terminal
            .draw(|frame| {
                let area = frame.size();
                frame.render_widget(text.block(block), area);
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
