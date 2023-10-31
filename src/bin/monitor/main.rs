use std::thread::sleep;
use std::time::Duration;

use cg::dummy_hdm::DummyHdm;
use cg::hardware_data_manager::HardwareDataManager;

fn main() {
    let mut hdm = DummyHdm::new();

    hdm.set_blockcount(4);

    let mut empty_polls = 0;
    while empty_polls < 2000 {
        match hdm.next() {
            Some(u) => println!("got {:?}", u),
            None => empty_polls += 1,
        }
        sleep(Duration::from_secs_f32(0.001));
    }
    hdm.stop();
}
