//! TODO

use crate::hardware_data_manager::{HardwareDataManager, Id, Update};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

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
    accumulated_updates: HashMap<(Id, Id), Update>,
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
        // Iterate through the hdm and collect all of the updates. Since `hdm_handle`
        // is an `Rc<RefCell<T>>`, and we want to mutate it, we use `.borrow_mut()` to
        // convert it into an &mut T (this is when the borrow checker runs). Then we
        // use `.by_ref()` so that we don't consume the iterator itself, just
        // the elements it iterates over.
        for update in self.hdm_handle.borrow_mut().by_ref() {
            // Insert those updates into the hash table, overwriting
            // exiting (and therefore older) entries
            self.accumulated_updates
                .insert((update.src, update.dst), update);
        }
        // Return a copy of the most recent updates, in a Vec rather than a HashMap
        self.accumulated_updates.values().cloned().collect()
    }
}
