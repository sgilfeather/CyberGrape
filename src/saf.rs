//! A safe api into the Spatial Audio Framework.

use std::ptr::{addr_of_mut, null_mut};

use crate::saf_raw;

use libc::c_void;

/// A Binauralizer is anything that can take an array of sound buffers, paired
/// with their associated metadata, and return a pair of freshly allocated
/// buffers representing the mixed stereo audio.
pub trait Binauralizer {
    // Our goal here is to implement process: 
    fn process(&self, buffers: &[(BufferMetadata, &[f32])]) -> (Vec<f32>, Vec<f32>);
}

/// The metadata associated with an audio stream. Includes the buffer's angular
/// position, range, and gain.
pub struct BufferMetadata {
    azmuth: f32,
    elevation: f32,
    range: f32,
    // gain: amount of amplification applied to a signal
    // ratio between the input volume and the output volume
    gain: f32,
    
}

/// Impl of Binauralizer that uses SAF's BinauralizerNF (Near Field)
pub struct BinauralizerNF {
    // stores C-style BinauralizerNF object, for use in libsaf
    h_bin: *mut c_void,     
}

impl BinauralizerNF for Binauralizer {
    /// Creates a new BinauralizerNF, initialized with [ TODO ]
    fn new() -> Self {
        let mut h_bin = null_mut();
        unsafe {
            saf_raw::BinauralizerNF_create(addr_of_mut!(h_bin));
        }

        // TODO 0. Configure hbin with num of inputs, direction, gain, range
        BinauralizerNF { h_bin }

        // may want to use binauraliserNF_setSourceDist_m() to set
        // range 
    }


    /// Takes a slice of audio data buffers, each paired with metadata encoding
    /// their location, range, and gain. Returns a pair of vectors containing
    /// the mixed binaural audio.
    fn process(&self, buffers: &[(BufferMetadata, &[f32])]) -> (Vec<f32>, Vec<f32>) {
        // TODO 1. Convert each slice in buffers to a raw pointer


        // TODO 2. Allocate output Vecs and create raw pointers to them 


        // TODO 3. Call binauraliserNF_process with correct args


        // TODO 4. Return output Vecs!
    }
}


impl Default for BinauralizerNF {
    fn default() -> Self {
        Self::new()
    }
}

/// Frees memory associated with BinauralizerNF struct
impl Drop for BinauralizerNF {
    fn drop(&mut self) {
        unsafe {
            saf_raw::BinauralizerNF_destroy(addr_of_mut!(self.h_bin));
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
}
