use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Chart, Dataset, GraphType},
    Frame, Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use cg::Point;

type PointGenerator = Box<dyn FnMut() -> Vec<Point>>;

struct App {
    orig_points_generator: PointGenerator,
    new_points_generator: PointGenerator,
    orig_points: Vec<(f64, f64)>,
    new_points: Vec<(f64, f64)>,
}

impl App {
    fn new(orig_points_generator: PointGenerator, new_points_generator: PointGenerator) -> App {
        App {
            orig_points_generator,
            new_points_generator,
            orig_points: vec![],
            new_points: vec![],
        }
    }

    fn on_tick(&mut self) {
        self.orig_points = (self.orig_points_generator)()
            .iter()
            .map(|&Point { x, y }| (x, y))
            .collect();
        self.new_points = (self.new_points_generator)()
            .iter()
            .map(|&Point { x, y }| (x, y))
            .collect();
    }
}

pub fn engage_gui(
    orig_points_generator: PointGenerator,
    new_points_generator: PointGenerator,
) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let app = App::new(orig_points_generator, new_points_generator);
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    // Padding added to the bounds of the chart
    let padding = 2.0;

    // Collect all the x and y values from orig_points and new_points
    let all_x = app.orig_points.iter().map(|(x, _)| x);
    let all_y = app.orig_points.iter().map(|(_, y)| y);

    // Compute lower and upper bounds for the chart
    let x_bounds = [
        all_x.clone().fold(f64::INFINITY, |a, b| a.min(*b)) - padding,
        all_x.clone().fold(f64::NEG_INFINITY, |a, b| a.max(*b)) + padding,
    ];
    let y_bounds = [
        all_y.clone().fold(f64::INFINITY, |a, b| a.min(*b)) - padding,
        all_y.clone().fold(f64::NEG_INFINITY, |a, b| a.max(*b)) + padding,
    ];

    let chart = Chart::new(vec![
        Dataset::default()
            .name("Original")
            .marker(symbols::Marker::Dot)
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(Color::Cyan))
            .data(&app.orig_points),
        Dataset::default()
            .name("Calculated")
            .marker(symbols::Marker::Dot)
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(Color::Red))
            .data(&app.new_points),
    ])
    .block(Block::default().title("Chart"))
    .x_axis(Axis::default().bounds(x_bounds))
    .y_axis(Axis::default().bounds(y_bounds));

    f.render_widget(chart, f.size());
}
