//! The various functions and utilities that wrap around [ratatui](https://docs.rs/ratatui/latest/ratatui/)
//! to make our project look nice.

mod device_selector;
mod error;
mod fold_until_stop;

pub use device_selector::device_selector;
pub use error::GrapeGuiError;
pub use fold_until_stop::fold_until_stop;
