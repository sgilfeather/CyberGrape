//! TODO

use clap::Parser;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use cybergrape::{
    args::{
        CommandTask::{Binaural, Serial},
        GrapeArgs,
    },
    hardware_message_decoder::HardwareEvent,
    hdm::Hdm,
    saf::BinauraliserNF,
    update_accumulator::UpdateAccumulator,
};

use hound::WavReader;
use log::{debug, info, warn};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListState},
};
use serial2::SerialPort;
use std::{
    cell::RefCell,
    collections::BinaryHeap,
    io::{self, stdout},
    rc::Rc,
    str::{self, FromStr},
};

// Example:
// cargo run --bin cybergrape --
//                            --samp    44100
//                            --update  40 binaural
//                            -n        2
//                            --out     outfile.wav
//                            --gains   1 1
//                            --ranges  3 4
//                            --files   x.wav y.wav

fn main() {
    env_logger::init();
    let args = GrapeArgs::parse();

    // logic to parse commandline arguments for serial vs binaural
    let SAMP_RATE: f32 = args.samp_rate;
    let UPDATE_RATE: f32 = args.update_rate;

    let cmd = args.command;
    match cmd {
        Binaural(BinauralCommand) => {
            let NUM_FILES: u32 = BinauralCommand.num_files;
            let OUTFILE: String = BinauralCommand.outfile;
            let INFILE_SAMPLES: Vec<Vec<f32>> = hound_reader(BinauralCommand.filenames);
            let INFILE_GAINS: Vec<f32> = BinauralCommand.gains;
            let INFILE_RANGES: Vec<f32> = BinauralCommand.ranges;
        }

        Serial(SerialCommand) => {
            let OUTFILE: String = SerialCommand.outfile;
        }
    };

    let available_ports = SerialPort::available_ports().expect("Failed to get available ports");

    enable_raw_mode().unwrap();
    stdout().execute(EnterAlternateScreen).unwrap();
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout())).unwrap();
    terminal.clear().unwrap();

    let mut selected_item = 0;
    let mut selected_port = None;

    loop {
        terminal
            .draw(|frame| {
                let port_names = available_ports.iter().map(|p| p.to_string_lossy());
                let list = List::new(port_names)
                    .block(
                        Block::default()
                            .title("Select Your Device")
                            .borders(Borders::ALL),
                    )
                    .style(Style::default().fg(Color::White))
                    .highlight_symbol(">>");

                let area = frame.size();
                let mut state = ListState::default().with_selected(Some(selected_item));
                frame.render_stateful_widget(list, area, &mut state);
            })
            .unwrap();
        if event::poll(std::time::Duration::from_millis(16)).unwrap() {
            if let event::Event::Key(key) = event::read().unwrap() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Down => {
                            selected_item = (selected_item + 1) % available_ports.len();
                        }
                        KeyCode::Up => {
                            selected_item =
                                (selected_item + available_ports.len() - 1) % available_ports.len();
                        }
                        KeyCode::Enter => {
                            selected_port = Some(selected_item);
                            break;
                        }
                        KeyCode::Char('q') => break,
                        _ => {}
                    }
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
    terminal.show_cursor().unwrap();

    if selected_port.is_none() {
        return;
    }

    let device_name = available_ports[selected_port.unwrap()].to_string_lossy();
    println!("Selected device: {}", device_name);

    // Ask user for the device name
    // println!("Available devices:");
    // for port in available_ports {
    //     println!("\t{}", port.to_string_lossy());
    // }
    // println!("Enter the device name: ");
    // let mut device_name = String::new();
    // io::stdin()
    //     .read_line(&mut device_name)
    //     .expect("Failed to read line");

    // Try to open the requested port and set its read timeout to infinity
    // (well, about 584,942,417,355 years, which is close enough)
    let mut port = SerialPort::open(device_name.trim(), 115200).expect("Failed to open port");
    port.set_read_timeout(std::time::Duration::MAX)
        .expect("Failed to set read timeout");

    let hdm = Rc::new(RefCell::new(Hdm::new()));
    let mut accumulator = UpdateAccumulator::new(hdm.clone());

    // Read from the port and print the received data
    let mut buffer = [0; 256];
    let mut read_buf = Vec::new();

    loop {
        let read_len = port.read(&mut buffer).expect("Device disconnected");

        for &c in buffer.iter().take(read_len) {
            read_buf.push(c);
            if c == b'\n' {
                match str::from_utf8(&read_buf) {
                    Ok(s) => match HardwareEvent::from_str(s) {
                        Ok(HardwareEvent::UUDFEvent(e)) => {
                            debug!("Received {:#?}, adding to HDM", e);
                            hdm.borrow_mut().add_update(e);
                        }
                        Ok(HardwareEvent::UUDFPEvent(ep)) => {
                            debug!("Received {:#?}", ep);
                        }
                        Err(e) => {
                            warn!("Was unable to parse hardware message: {}", e);
                        }
                    },
                    // Often happens at the beginning of transmission when
                    // there is still garbage in the hardware buffer
                    Err(e) => {
                        warn!("Failed to decode utf-8: {:?}", e);
                    }
                }
                info!("Hdm has {:#?}", accumulator.get_status());

                read_buf.clear();
            }
        }
    }
}

///
/// This function, given a Vector of filenames, uses hound to read the audio
/// data into a 2D vector, where each vector represents the audio file data.
///
fn hound_reader(filenames: Vec<String>) -> Vec<Vec<f32>> {
    let mut all_samples: Vec<Vec<f32>> = vec![];

    for file in filenames {
        let mut reader = WavReader::open(file).unwrap();

        // collect wav file data into Vec of interleaved f32 samples
        let samples = reader
            .samples::<i32>()
            .map(|x| x.unwrap() as f32)
            .collect::<Vec<_>>();

        all_samples.push(samples);
    }

    all_samples
}
