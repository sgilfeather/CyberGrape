//! TODO

use std::io;

use serial2::SerialPort;

fn main() {
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

    // Read from the port and print the received data
    let mut buffer = [0; 256];
    let mut read_buf = Vec::new();
    loop {
        let read_len = port.read(&mut buffer).expect("Device disconnected");

        for &c in buffer.iter().take(read_len) {
            read_buf.push(c);
            if c == b'\n' {
                print!("Received: {}", String::from_utf8_lossy(&read_buf));
                read_buf.clear();
            }
        }
    }
}
