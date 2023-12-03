use crate::hardware_data_manager::{HardwareDataManager, Id, Update};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct UpdateAccumulator<Hdm>
where
    Hdm: HardwareDataManager,
{
    hdm_handle: Rc<RefCell<Hdm>>,
    accumulated_updates: HashMap<(Id, Id), Update>,
}

impl<Hdm> UpdateAccumulator<Hdm>
where
    Hdm: HardwareDataManager,
{
    pub fn new(hdm_handle: Rc<RefCell<Hdm>>) -> Self {
        Self {
            hdm_handle,
            accumulated_updates: HashMap::new(),
        }
    }
    pub fn get_status(&mut self) -> Vec<Update> {
        for update in self.hdm_handle.borrow_mut().by_ref() {
            self.accumulated_updates
                .insert((update.src, update.dst), update);
        }
        self.accumulated_updates.values().cloned().collect()
    }
}
