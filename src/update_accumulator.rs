use crate::hardware_data_manager::{HardwareDataManager, Id, Update};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct UpdateAccumulator<Hdm>
where
    Hdm: HardwareDataManager,
{
    hdm_handle: Arc<Mutex<Hdm>>,
    accumulated_updates: HashMap<(Id, Id), Update>,
}

impl<Hdm> UpdateAccumulator<Hdm>
where
    Hdm: HardwareDataManager,
{
    pub fn new(hdm_handle: Arc<Mutex<Hdm>>) -> Self {
        Self {
            hdm_handle,
            accumulated_updates: HashMap::new(),
        }
    }
    pub fn get_status(&mut self) -> Vec<Update> {
        // ISSUE 38
        todo!()
    }
}
