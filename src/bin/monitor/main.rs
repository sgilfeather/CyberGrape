mod gui;

use std::cell::RefCell;
use std::rc::Rc;

use cg::dummy_hdm::DummyHdm;
use cg::localizer::localize_points;
use cg::update_accumulator::UpdateAccumulator;
use gui::engage_gui;

fn main() {
    let hdm = DummyHdm::builder()
        .num_points(10)
        .range(5.0)
        .noise(0.25)
        .build();
    let hdm_mtx = Rc::new(RefCell::new(hdm));
    let hdm = hdm_mtx.clone();

    let update_acc_hdm_handle = hdm_mtx.clone();
    let mut update_acc = UpdateAccumulator::new(update_acc_hdm_handle);

    let debug_hdm_handle = hdm_mtx.clone();

    let _ = engage_gui(
        Box::new(move || debug_hdm_handle.borrow().get_debug_locations()),
        Box::new(move || localize_points(&update_acc.get_status())),
    );

    hdm.borrow_mut().stop();
}
