pub mod args;
pub mod component;
pub mod device_selector;
pub mod dummy_hdm;
pub mod hardware_data_manager;
pub mod hardware_message_decoder;
pub mod hdm;
pub mod hound_helpers;
pub mod localizer;
pub mod saf;
mod saf_raw;
pub mod spatial_data_format;
pub mod sphericalizer;
pub mod time_domain_buffer;
pub mod update_accumulator;

use std::fmt;

// `Copy` is what we call types that do not need to be borrowed. This is very
// similar to pass-by-value in C/C++. Basic types (integers, floats, etc.) are
// all `Copy`. Fancier things like `String`s are "not `Copy`" because we want
// to borrow those usually. We cannot manually implement `Copy`, it is just a
// signal to the compiler. Any type that is `Copy` must also implement `Clone`.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    #[allow(dead_code)]
    pub fn abs_dist(&self, other: &Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}

// `Debug` is for dirty, exhaustive, and specific output; the kind that the
// compiler can come up with. If we want something that looks nicer, we use
// another trait, `Display`. This one cannot be `#[derive()]`d, since asthetics
// are not something the compiler cares about, so we implement it ourselves.
impl fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.3}, {:.3})", self.x, self.y)
    }
}
