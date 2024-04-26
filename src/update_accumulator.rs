//! TODO

use crate::hardware_data_manager::{HardwareDataManager, Id, Update};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

const BUFFER_SIZE: usize = 5;

/// The `UpdateAccumulator` consumes updates from a `HardwareDataManager`, and
/// accumulates them. It can be queried for the most recent updates using `.get_status()`.
// The <Hdm> means that we are allowed to use `Hdm` as a type within `UpdateAccumulator`.
pub struct UpdateAccumulator<Hdm>
where
    // Then this binding ensures that `Hdm` implements `HardwareDataManager`.
    Hdm: HardwareDataManager,
{
    // `Rc` means this is a "reference-counted" smart pointer, and `RefCell` means we
    // are going to enforce the borrow checking rules at runtime instead of
    // compile time. This way we can keep references to the HDM in several scopes.
    hdm_handle: Arc<Mutex<Hdm>>,

    // A HashMap mapping `(Id, Id)` pairs to `Update`s.
    accumulated_updates: HashMap<(Id, Id), VecDeque<Update>>,
}

// We see `Hdm` in three places here. First, it is declared as a type for use
// in the impl block. Second, we say that the impl block applies to `UpdateAccumulator`s
// that use the `Hdm` type. Finally, we say that `Hdm` must implement the
// `HardwareDataManager` trait.
impl<Hdm> UpdateAccumulator<Hdm>
where
    Hdm: HardwareDataManager,
{
    /// Instantiates a new `UpdateAccumulator` attached to a `Hdm`
    pub fn new(hdm_handle: Arc<Mutex<Hdm>>) -> Self {
        Self {
            hdm_handle,
            accumulated_updates: HashMap::new(),
        }
    }

    /// Returns a `Vec` contatining the most recent `Update`s for all pairs
    /// of blocks. Essentially, the most updated data available.
    pub fn get_status(&mut self) -> Vec<Update> {
        for update in self.hdm_handle.lock().unwrap().by_ref() {
            self.accumulated_updates
                .entry((update.src, update.dst))
                .and_modify(|v| v.push_back(update.clone()))
                .or_insert_with(|| VecDeque::from(vec![update.clone()]));
        }

        // Return a copy of the most recent updates, in a Vec rather than a HashMap
        let res = self
            .accumulated_updates
            .values()
            .map(|v| {
                let taken = v.iter().rev().take(BUFFER_SIZE);
                let len = taken.len() as f64;
                let sum = taken
                    .cloned()
                    .reduce(|l, r| Update {
                        elv: l.elv + r.elv,
                        azm: l.azm + r.azm,
                        ..l
                    })
                    .expect("There should be some elements here");

                Update {
                    elv: sum.elv / len,
                    azm: sum.azm / len,
                    ..sum
                }
            })
            .collect();

        for v in self.accumulated_updates.values_mut() {
            if v.len() > BUFFER_SIZE {
                v.pop_front();
            }
        }

        res
    }
}
