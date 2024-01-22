//! A safe api into the Spatial Audio Framework.

use std::ptr::{addr_of_mut, null_mut};

use crate::saf_raw;

use libc::c_void;

/// A Binauraliser is anything that can take an array of sound buffers, paired
/// with their associated metadata, and return a pair of freshly allocated
/// buffers representing the mixed stereo audio.
pub trait Binauraliser {
    fn process(&self, buffers: &[(BufferMetadata, &[f32])]) -> (Vec<f32>, Vec<f32>);
}

/// The metadata associated with an audio stream. Includes the buffer's angular
/// position, range, and gain.
pub struct BufferMetadata {
    azmuth: f32,
    elevation: f32,
    range: f32,
    gain: f32,
}

// Stub implementation of our binauraliser using SAF
pub struct BinauraliserNF {
    h_bin: *mut c_void,
}

impl BinauraliserNF {
    pub fn new() -> Self {
        let mut h_bin = null_mut();
        unsafe {
            saf_raw::binauraliserNF_create(addr_of_mut!(h_bin));
        }
        BinauraliserNF { h_bin }
    }
}

impl Default for BinauraliserNF {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BinauraliserNF {
    fn drop(&mut self) {
        unsafe {
            saf_raw::binauraliserNF_destroy(addr_of_mut!(self.h_bin));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
