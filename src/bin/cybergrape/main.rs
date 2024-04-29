//! TODO

use clap::Parser;
use cybergrape::{
    args::{
        CommandTask::{Binaural, Serial},
        GrapeArgs,
    },
    gui,
    hardware_message_decoder::HardwareEvent,
    hdm::Hdm,
    hound_helpers::{hound_reader, write_stereo_output},
    saf::{Binauraliser, BinauraliserNF},
    spatial_data_format::{GrapeFile, GrapeTag},
    sphericalizer::Sphericalizer,
    time_domain_buffer::TDBufMeta,
    update_accumulator::UpdateAccumulator,
    TransposableIter,
};

use log::{debug, error, info, warn};
use serial2::SerialPort;
use spin_sleep::sleep;
use std::{
    str::{self, FromStr},
    sync::{Arc, Mutex},
    thread::spawn,
    time::Duration,
};

const BAUD_RATE: u32 = 115200;

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
    let update_rate = args.update_rate;

    let cmd = args.command;

    let (num_tags, outfile, audio_settings) = match cmd {
        Binaural(binaural_command) => (
            binaural_command.num_files,
            binaural_command.outfile,
            Some((
                hound_reader(binaural_command.filenames),
                binaural_command.gains,
                binaural_command.ranges,
                binaural_command.samp_rate,
            )),
        ),
        Serial(serial_command) => (
            serial_command.num_tags,
            // outfile is common to both commands, though it means slightly different things
            serial_command.outfile,
            // the serial command doesn't have any audio samples and doesn't need gain/range info
            None,
        ),
    };

    // Figure out what serial port our antena box is on
    let available_ports = SerialPort::available_ports()?;
    let selected_port_opt = gui::device_selector(available_ports)?;
    let selected_port = match selected_port_opt {
        Some(port) => port,
        None => {
            error!("Port selection failed");
            return Ok(());
        }
    };

    // Try to open the requested port and set its read timeout to infinity
    // (well, about 584,942,417,355 years, which is close enough)
    let mut port = SerialPort::open(selected_port, BAUD_RATE).expect("Failed to open port");
    port.set_read_timeout(std::time::Duration::MAX)
        .expect("Failed to set read timeout");

    let hdm = Arc::new(Mutex::new(Hdm::new()));

    listen_on_port(port, hdm.clone());

    if let Some((sound_data, gains, ranges, sample_rate)) = audio_settings {
        let sphericalizer = Sphericalizer::new(gains.into_iter().zip(ranges).collect());

        let shortest = sound_data
            .iter()
            .map(|v| v.len())
            .max()
            .expect("should have some files");
        let len_shortest = (shortest / sample_rate) - 1;

        let num_updates_needed = len_shortest * update_rate;
        let samples_per_update = sample_rate / update_rate;

        let mut td_buf = TDBufMeta::new(num_tags);
        let time_delta = Duration::from_secs(1).div_f64(update_rate as f64);

        info!("shortest: {:#?}", shortest);
        info!("len_shortest: {:#?}", len_shortest);
        info!("num_updates_needed: {:#?}", num_updates_needed);
        info!("samples_per_update: {:#?}", samples_per_update);
        info!("time_delta: {:#?}", time_delta);

        info!("gathering data");

        let mut accumulator = UpdateAccumulator::new(hdm.clone());
        for _ in 0..num_updates_needed {
            if let Some(update) = sphericalizer.query(&mut accumulator) {
                td_buf.add(update)
            }
            sleep(time_delta);
        }
        let spatial_data = td_buf.dump();
        let mut binauraliser = BinauraliserNF::new();

        let mut out_left: Vec<f32> = vec![];
        let mut out_right: Vec<f32> = vec![];

        info!("post processing");

        for (i, metadata) in spatial_data.into_iter().enumerate() {
            let sound_start = i * samples_per_update;
            let sound_stop = (i + 1) * samples_per_update;
            let sound_slices = sound_data
                .iter()
                .map(|v| &v[sound_start..sound_stop])
                .collect::<Vec<_>>();

            assert_eq!(metadata.len(), sound_slices.len());

            let tagged_buffers = metadata.into_iter().zip(sound_slices).collect::<Vec<_>>();

            let (mut new_left, mut new_right) = binauraliser.process(&tagged_buffers[..]);

            out_left.append(&mut new_left);
            out_right.append(&mut new_right);
        }

        info!("writing the output file");

        write_stereo_output(out_left, out_right, outfile);
    } else {
        let th_hdm = hdm.clone();
        let sphericalizer = Sphericalizer::new(vec![(1.0, 1.0); num_tags]);

        let td_buf = TDBufMeta::new(num_tags);
        let time_delta = Duration::from_secs(1).div_f64(update_rate as f64);

        let accumulator = UpdateAccumulator::new(th_hdm);

        let (buf, _) = gui::fold_until_stop((td_buf, accumulator), move |(mut buf, mut acc)| {
            if let Some(update) = sphericalizer.query(&mut acc) {
                buf.add(update)
            }
            sleep(time_delta);
            (buf, acc)
        })?;

        let data = buf.dump();

        let grape_file_builder = GrapeFile::builder().set_samplerate(update_rate as u64);

        let grape_file = data
            .transpose()
            .fold(grape_file_builder, |b, v| {
                let azms: Vec<f32> = v.iter().map(|e| e.azimuth).collect();
                let elvs: Vec<f32> = v.iter().map(|e| e.elevation).collect();
                b.add_stream(&azms, GrapeTag::Azimuth)
                    .add_stream(&elvs, GrapeTag::Elevation)
            })
            .build()?;

        grape_file.to_path(outfile)?;
    }

    Ok(())
}

fn listen_on_port(port: SerialPort, hdm: Arc<Mutex<Hdm>>) {
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
}
