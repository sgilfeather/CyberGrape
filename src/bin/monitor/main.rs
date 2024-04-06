//! This executable spins up the dummy hardware data manager and tries to localize
//! the updates it receives. It then displays the calculated locations on top of the
//! original locations that were used to generate the updates so that we can assess
//! the performance of the localization algorithm.

mod gui;

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use cybergrape::dummy_hdm::DummyHdm;
use cybergrape::localizer::localize_points;
use cybergrape::update_accumulator::UpdateAccumulator;
use gui::engage_gui;

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

    // Configure, instantiate, and start the dummy HDM.
    let hdm = DummyHdm::builder()
        .num_points(10)
        .range(5.0)
        .noise(0.25)
        .build();

    // We're going to need a few references active to this HDM at once, so we
    // wrap it in a RefCell to indicate that we want to enforce the borrow checking
    // rules at _runtime_ rather than compile-time. The Rc allows us to keep
    // references to the data in several different scopes.
    //
    // This is called the **interior mutability pattern**.
    let hdm_rf = Rc::new(RefCell::new(hdm));

    // Now we're going to shadow over the hdm variable with a reference so that
    // we don't accidentially do something funky with the original thing.
    let hdm = hdm_rf.clone();

    // Instantiate an UpdateAccumulator with a pointer to the HDM.
    let update_acc_hdm_handle = hdm_rf.clone();
    let mut update_acc = UpdateAccumulator::new(update_acc_hdm_handle);

    let debug_hdm_handle = hdm_rf.clone();

    // Ok now this is the wonky bit. We're going to define closures to pass into
    // this function. The || indicates that this is a closure that takes no arguments
    // and move indicates that captured variables will be _moved_ into the scope
    // of the function, rather than being borrowed.
    //
    // So, we move the debug_hdm_handle into the first closure, then .borrow()
    // to turn the Rc<RefCell<T>> into an &T, which we can then call the
    // .get_debug_locations() on.
    //
    // Remember that those closures are **not** being run immediately, they are
    // instead run roughly every quarter second by the GUI.
    let _ = engage_gui(
        Box::new(move || debug_hdm_handle.borrow().get_debug_locations()),
        Box::new(move || localize_points(&update_acc.get_status())),
    );

    // Once the gui terminates, we take a mutable referene to the hdm and stop it.
    // .borrow_mut() takes the Rc<RefCell<T>> and turns it into an &mut T.
    hdm.borrow_mut().stop();
}
