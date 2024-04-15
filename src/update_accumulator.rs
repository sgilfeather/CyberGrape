//! TODO

use crate::hardware_data_manager::{HardwareDataManager, Id, Update};
use std::{
    cell::RefCell,
    collections::{hash_map, HashMap, VecDeque},
    rc::Rc,
};

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
    hdm_handle: Rc<RefCell<Hdm>>,

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
    pub fn new(hdm_handle: Rc<RefCell<Hdm>>) -> Self {
        Self {
            hdm_handle,
            accumulated_updates: HashMap::new(),
        }
    }

    /// Returns a `Vec` contatining the most recent `Update`s for all pairs
    /// of blocks. Essentially, the most updated data available.
    pub fn get_status(&mut self) -> Vec<Update> {
        for update in self.hdm_handle.borrow_mut().by_ref() {
            if let hash_map::Entry::Vacant(e) =
                self.accumulated_updates.entry((update.src, update.dst))
            {
                let mut v = VecDeque::new();
                v.push_back(update.clone());
                e.insert(v);
            } else {
                let v = self
                    .accumulated_updates
                    .get_mut(&(update.src, update.dst))
                    .expect("This really should exist, we just checked");
                v.push_back(update);
            }
        }

        // Return a copy of the most recent updates, in a Vec rather than a HashMap
        let res = self
            .accumulated_updates
            .values()
            .map(|v| {
                let taken = v.iter().rev().take(50);
                let len = taken.len() as f64;
                let sum = taken
                    .cloned()
                    .reduce(|l, r| Update {
                        elv: l.elv + r.elv,
                        azm: l.azm + r.azm,
                        ..l
                    })
                    .unwrap_or(v[0].clone());

                Update {
                    elv: sum.elv / len,
                    azm: sum.azm / len,
                    ..sum
                }
            })
            .collect();

        for v in self.accumulated_updates.values_mut() {
            let len = v.len();
            if len > 50 {
                let to_drop = len - 50;
                v.drain(0..to_drop);
            }
        }

        res
    }
}
