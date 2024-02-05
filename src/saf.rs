//! A safe api into the Spatial Audio Framework.

use std::ptr::{addr_of_mut, null, null_mut};
use std::slice;

use crate::saf_raw;

use hound::Sample;
use libc::c_void;

// Sets all audio channel distances to 1 meter—— tretch goal to specify per channel
const DIST_DEFAULT: f32 = 1.0;
const SAMP_RATE: i32 = 44100;
const NUM_OUT_CHANNELS: usize = 2;
const RAD_TO_DEGREE: f32 = 180.0 / std::f32::consts::PI;


/// A Binauraliser is anything that can take an array of sound buffers, paired
/// with their associated metadata, and return a pair of freshly allocated
/// buffers representing the mixed stereo audio.
pub trait Binauraliser {
    fn new() -> Self;
    fn process(&self, buffers: &[(BufferMetadata, &[f32])]) -> (Vec<f32>, Vec<f32>);
}

/// The metadata associated with an audio stream. Includes the buffer's angular
/// position, range, and gain.
#[derive(Clone)]
#[derive(Copy)]
pub struct BufferMetadata {
    azmuth: f32,
    elevation: f32,
    range: f32,
    // gain: amount of amplification applied to a signal
    // ratio between the input volume and the output volume
    gain: f32,
}

/// Impl of Binauraliser that uses SAF's BinauraliserNF (Near Field)
pub struct BinauraliserNF {
    // stores C-style BinauraliserNF object, for use in libsaf
    h_bin: *mut c_void,
}

impl Binauraliser for BinauraliserNF {
    /// Creates a new BinauraliserNF, initialized with [ TODO ]
    fn new() -> Self {
        let mut h_bin = null_mut();
        unsafe {
            saf_raw::binauraliserNF_create(addr_of_mut!(h_bin));

            // initialize sample rate
            saf_raw::binauraliserNF_init(h_bin, SAMP_RATE);

            // initialize codec variables, whatever those are
            saf_raw::binauraliserNF_initCodec(h_bin);
        }

        BinauraliserNF { h_bin }
    }


    /// Takes a slice of audio data buffers, each paired with metadata encoding
    /// their location, range, and gain. Returns a pair of vectors containing
    /// the mixed binaural audio.
    /// 
    /// Invariant: All input buffers must be the same length
    fn process(&self, buffers: &[(BufferMetadata, &[f32])]) -> (Vec<f32>, Vec<f32>) {
        // convert each slice in buffers to a raw pointer
        let num_channels: usize = buffers.len();
        let num_samples: usize = buffers[0].1.len();
        
        // allocate input and output buffers for process() call
        let mut raw_input_ptrs: Vec<*const f32> = vec![null();num_channels];
        let mut raw_output_ptrs: Vec<*mut f32> = vec![null_mut();NUM_OUT_CHANNELS];

        for i in 0..NUM_OUT_CHANNELS {
            raw_output_ptrs[i] = vec![0.0;num_samples].as_mut_ptr();
        }

        // allocate output Vecs and create raw pointers to them 
        let output_vec_1: Vec<f32>;
        let output_vec_2: Vec<f32>;

        unsafe {
            for (i, (metadata, audio_data)) in buffers.into_iter().enumerate() {
                // store raw pointer for channel in raw_data_ptrs
                raw_input_ptrs[i] = audio_data.as_ptr();

                // set distance, azimuth, and elevation for each channel
                saf_raw::binauraliserNF_setSourceDist_m(self.h_bin, i as i32, DIST_DEFAULT);
                saf_raw::binauraliser_setSourceAzi_deg(self.h_bin, i as i32, metadata.azmuth * RAD_TO_DEGREE);
                saf_raw::binauraliser_setSourceElev_deg(self.h_bin, i as i32, metadata.elevation * RAD_TO_DEGREE);
            }
            
            // call process() to convert to binaural audio
            saf_raw::binauraliserNF_process(
                self.h_bin,
                raw_input_ptrs.as_slice().as_ptr(),  // N inputs x K samples
                raw_output_ptrs.as_slice().as_ptr(), // N inputs x K samples
                num_channels as i32,                 // N inputs
                NUM_OUT_CHANNELS as i32,             // N outputs
                num_samples as i32                   // K samples
            );

            // convert raw pointers updated by process() back to vectors
            output_vec_1 = slice::from_raw_parts(raw_output_ptrs[0], num_samples).to_vec();
            output_vec_2 = slice::from_raw_parts(raw_output_ptrs[1], num_samples).to_vec();
        }

        (output_vec_1, output_vec_2)
    }
}


impl Default for BinauraliserNF {
    fn default() -> Self {
        Self::new()
    }
}

/// Frees memory associated with BinauraliserNF struct
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

    /// Test BinauraliserNF constructor
    #[test]
    fn test_create_binauraliser() {
        BinauraliserNF::new();
    }

    #[test]
    // Test BinauraliserNF process
    fn test_process_null_data() {
        eprintln!("before new");
        let binauraliser_nf: BinauraliserNF = BinauraliserNF::new();
        const NUM_SAMPLES: usize = 4410000;
        const NUM_CHANNELS: usize = 4;

        let mock_meta_data: BufferMetadata = BufferMetadata {
            azmuth: 0.0,
            elevation: 0.0,
            range: 0.0,
            gain: 0.0
        };

        let mock_audio_data = vec![0.0 as f32; NUM_SAMPLES];
        let mock_buffers = [(mock_meta_data, mock_audio_data.as_slice()); NUM_CHANNELS];
        
        eprintln!("process call in test");
        let (left_vec, right_vec) = binauraliser_nf.process(mock_buffers.as_ref());
    
        println!("AHHHHHHHHH");
        println!("{:?}", left_vec);
        println!("{:?}", right_vec);
    }

    // Invalid input tests
    // buffers is empty
    // if several channel / audio buffers have different lengths
    // 
}
