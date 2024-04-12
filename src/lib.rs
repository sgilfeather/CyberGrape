pub mod dummy_hdm;
pub mod grape_block;
pub mod hardware_data_manager;
pub mod localizer;
pub mod saf;
mod saf_raw;
pub mod update_accumulator;

use std::any::Any;
use std::fmt::Display;

///
/// A stage in the CyberGrape pipeline, which performs a step of the data
/// aggregation, binauralization, or music playback process. All structs
/// that perform a processing step in the CyberGrape system must implement
/// Component, so that they can be integrated into the pipeline.
///
pub trait Component: Display {
    type InData;
    type OutData;
    /// Converts an input of type A into an output of type B
    fn convert(self: &Self, input: Self::InData) -> Self::OutData;
}

// Type alias for a generic Component with any InData and OutData type
type AnyComponent = dyn Component<InData = dyn Any, OutData = dyn Any>;

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
impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.3}, {:.3})", self.x, self.y)
    }
}
