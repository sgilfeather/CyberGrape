use crate::hardware_data_manager::{HardwareDataManager, Update};
use crate::hardware_message_decoder::{HardwareEvent, UUDFEvent, UUDFPEvent};

use std::collections::VecDeque;
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};

pub struct Hdm {
    msgs: Arc<Mutex<VecDeque<Update>>>,
}

impl Hdm {
    pub fn new() -> Self {
        Hdm {
            msgs: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn add_update(&self, event: UUDFEvent) {
        let new_update = Update {
            src: event.anchor_id as usize,
            dst: event.tag_id as usize,
            azm: event.angle_1 as f64 * (PI / 180.0),
            elv: event.angle_2 as f64 * (PI / 180.0),
        };

        self.msgs.lock().unwrap().push_front(new_update);
    }
}

impl Iterator for Hdm {
    type Item = Update;

    fn next(&mut self) -> Option<Self::Item> {
        self.msgs.lock().unwrap().pop_front()
    }
}

impl HardwareDataManager for Hdm {
    fn clear(&mut self) {
        self.msgs.lock().unwrap().clear();
    }
}
