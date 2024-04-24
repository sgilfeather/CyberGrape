//! TODO

use clap::Parser;
use cybergrape::{
    args::{
        CommandTask::{Binaural, Serial},
        GrapeArgs,
    },
    device_selector,
    hardware_message_decoder::HardwareEvent,
    hdm::Hdm,
    sphericalizer::Sphericalizer,
    time_domain_buffer::TDBufMeta,
    update_accumulator::UpdateAccumulator,
};

use hound::WavReader;
use log::{debug, info, warn};
use serial2::SerialPort;
use spin_sleep::sleep;
use std::{
    iter::zip,
    str::{self, FromStr},
    sync::{Arc, Mutex},
    thread::spawn,
    time::Duration,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = GrapeArgs::parse();

    // logic to parse commandline arguments for serial vs binaural
    let update_rate: f32 = args.update_rate;

    let cmd = args.command;

    let num_files;
    let _outfile;
    let _infile_samples;
    let infile_gains;
    let infile_ranges;

    match cmd {
        Binaural(binaural_command) => {
            num_files = Some(binaural_command.num_files);
            _outfile = Some(binaural_command.outfile);
            _infile_samples = Some(hound_reader(binaural_command.filenames));
            infile_gains = Some(binaural_command.gains);
            infile_ranges = Some(binaural_command.ranges);
        }

        Serial(serial_command) => {
            num_files = Some(2);
            _outfile = Some(serial_command.outfile);
            infile_gains = Some(vec![1.0, 1.0]);
            infile_ranges = Some(vec![1.0, 1.0]);
        }
    };

    let available_ports = SerialPort::available_ports().expect("Failed to get available ports");
    let selected_port_opt = device_selector::run_device_selector(available_ports)?;
    let selected_port = match selected_port_opt {
        Some(port) => port,
        None => return Ok(()),
    };

    // Try to open the requested port and set its read timeout to infinity
    // (well, about 584,942,417,355 years, which is close enough)
    let mut port = SerialPort::open(selected_port, 115200).expect("Failed to open port");
    port.set_read_timeout(std::time::Duration::MAX)
        .expect("Failed to set read timeout");

    let hdm = Arc::new(Mutex::new(Hdm::new()));
    let mut accumulator = UpdateAccumulator::new(hdm.clone());

    let _hdm_thread = spawn(move || {
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
                                hdm.lock().unwrap().add_update(e);
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
                    read_buf.clear();
                }
            }
        }
    });

    let mut td_buf = TDBufMeta::new(num_files.expect("should come from args") as usize);
    let time_delta = Duration::from_secs(1).div_f32(update_rate);
    let sphericalizer =
        Sphericalizer::new(zip(infile_gains.unwrap(), infile_ranges.unwrap()).collect());

    for _ in 0..10000 {
        if let Some(update) = sphericalizer.query(&mut accumulator) {
            td_buf.add(update)
        }
        sleep(time_delta);
    }

    info!("{:#?}", td_buf.dump());

    Ok(())
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
