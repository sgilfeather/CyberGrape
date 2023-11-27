mod gui;

use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use cg::dummy_hdm::DummyHdm;
use cg::hardware_data_manager::HardwareDataManager;
use cg::update_accumulator::UpdateAccumulator;
use cg::localizer::localize_points;
use gui::engage_gui;

fn main() {
    let hdm_mtx = Arc::new(Mutex::new(DummyHdm::new()));
    let hdm = hdm_mtx.clone();

    hdm.lock().unwrap().set_blockcount(4);

    let update_acc_hdm_handle = hdm_mtx.clone();
    let mut update_acc = UpdateAccumulator::new(update_acc_hdm_handle);

    let debug_hdm_hande = hdm_mtx.clone();

    let _ = engage_gui(
        Box::new(move || debug_hdm_hande.lock().unwrap().get_debug_locations()),
        Box::new(move || localize_points(&update_acc.get_status())),
    );

    let mut empty_polls = 0;
    while empty_polls < 2000 {
        match hdm.lock().unwrap().next() {
            Some(u) => println!("got {:?}", u),
            None => empty_polls += 1,
        }
        sleep(Duration::from_secs_f32(0.001));
    }
    hdm.lock().unwrap().stop();
}
