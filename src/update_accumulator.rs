use std::{sync::{Arc, Mutex}, collections::HashMap};
use crate::hardware_data_manager::{HardwareDataManager, Update, Id};

pub struct UpdateAccumulator<Hdm> where Hdm: HardwareDataManager {
    hdm_handle: Arc<Mutex<Hdm>>,
    accumulated_updates: HashMap<(Id, Id), Update>
}

impl <Hdm> UpdateAccumulator<Hdm> where Hdm: HardwareDataManager {
    pub fn new(hdm_handle: Arc<Mutex<Hdm>>) -> Self {
        Self {
            hdm_handle,
            accumulated_updates: HashMap::new(),
        }
    }
    pub fn get_status(&self) -> Vec<Update> {
        // ISSUE 38
        todo!()
    }
}

