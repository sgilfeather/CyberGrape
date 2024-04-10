//! This module places unsafe SAF rust bindings in the crate under their
//! own namespace.

// We need all of these so that the compiler doesn't get angry about the
// C naming conventions and unused functions.
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]
#![allow(rustdoc::broken_intra_doc_links)]

// Now we finally copy/paste in the bindings that were built by bindgen
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
