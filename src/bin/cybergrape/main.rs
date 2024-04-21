//! TODO

use cybergrape::hardware_message_decoder::HardwareEvent;
use cybergrape::hdm::Hdm;
use cybergrape::update_accumulator::UpdateAccumulator;

use log::{debug, info, warn};
use serial2::SerialPort;
use std::{cell::RefCell, io, rc::Rc, str, str::FromStr};

fn main() {
    env_logger::init();
    // Ask user for the device name
    let available_ports = SerialPort::available_ports().expect("Failed to get available ports");
    println!("Available devices:");
    for port in available_ports {
        println!("\t{}", port.to_string_lossy());
    }
    println!("Enter the device name: ");
    let mut device_name = String::new();
    io::stdin()
        .read_line(&mut device_name)
        .expect("Failed to read line");

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

                info!("{}[2J", 27 as char);
                for update in accumulator.get_status() {
                    info!("{}", update);
                }

                read_buf.clear();
            }
        }
    }
}
