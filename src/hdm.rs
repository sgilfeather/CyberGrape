//! The thread-safe buffer where we will store [`UUDFEvent`]s from the antennas.

use crate::hardware_data_manager::{HardwareDataManager, Update};
use crate::hardware_message_decoder::UUDFEvent;

use std::{
    collections::VecDeque,
    f64::consts::PI,
    sync::{Arc, Mutex},
};

/// A [`HardwareDataManager`] that simply acts as a thread-safe buffer where
/// we can store [`UUDFEvent`]s from the antennas.
#[derive(Debug, Default)]
pub struct Hdm {
    msgs: Arc<Mutex<VecDeque<Update>>>,
}

impl Hdm {
    /// Instantiae a new [`Hdm`] with a thread-safe buffer.
    pub fn new() -> Self {
        Hdm {
            msgs: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Insert a new [`UUDFEvent`] into the thread-safe buffer.
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
