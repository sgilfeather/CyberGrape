//! A safe api into the Spatial Audio Framework.

use std::ptr::{addr_of_mut, null, null_mut};
use std::slice;


use crate::saf_raw;

use hound::Sample;
use hound::WavReader;
use hound::WavSpec;
use hound::WavWriter;
use hound::SampleFormat;
use libc::c_void;

use std::io::{self};

// Sets all audio channel distances to 1 meter—— stretch goal to specify per channel
const DIST_DEFAULT: f32 = 1.0;
const SAMP_RATE: i32 = 44100;
const NUM_OUT_CHANNELS: usize = 2;
const FRAME_SIZE: usize = 128;

const RAD_TO_DEGREE: f32 = 180.0 / std::f32::consts::PI;

const FILE_PATH: &'static str = "/Users/Skylar.Gilfeather/Desktop/were_the_rats.wav";
const OUT_FILE_PATH: &'static str = "/Users/Skylar.Gilfeather/Desktop/were_the_rats_out.wav";


/// A Binauraliser is anything that can take an array of sound buffers, paired
/// with their associated metadata, and return a pair of freshly allocated
/// buffers representing the mixed stereo audio.
pub trait Binauraliser {
    fn new() -> Self;
    fn process(&mut self, buffers: &[(BufferMetadata, &[f32])]) -> (Vec<f32>, Vec<f32>);
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

            saf_raw::binauraliser_setUseDefaultHRIRsflag(h_bin, 1);            
        }

        BinauraliserNF { h_bin }
    }


    /// Takes a slice of audio data buffers, each paired with metadata encoding
    /// their location, range, and gain. Returns a pair of vectors containing
    /// the mixed binaural audio.
    /// 
    /// Invariant: All input buffers must be the same length, of 128
    fn process(&mut self, buffers: &[(BufferMetadata, &[f32])]) -> (Vec<f32>, Vec<f32>) {
    
        // convert each slice in buffers to a raw pointer
        let num_channels: usize = buffers.len();
        
        // allocate input and output buffers for process() call
        let mut raw_input_ptrs: Vec<*const f32> = vec![null(); num_channels];
        let mut raw_output_ptrs: Vec<*mut f32> = vec![null_mut(); NUM_OUT_CHANNELS];


        for i in 0..NUM_OUT_CHANNELS {
            let mut samp_vec = vec![0.0; FRAME_SIZE];
            raw_output_ptrs[i] = samp_vec.as_mut_ptr();
            // ensure that elems of raw_output_ptrs vec aren't freed during process
            samp_vec.leak();
        }

        // // allocate output Vecs and create raw pointers to them 
        let output_vec_1: Vec<f32>;
        let output_vec_2: Vec<f32>;

        unsafe {
            saf_raw::binauraliser_setNumSources(self.h_bin, num_channels as i32);

            for (i, &(metadata, audio_data)) in buffers.iter().enumerate() {
                // store raw pointer for channel in raw_data_ptrs
                raw_input_ptrs[i] = audio_data.as_ptr();

                eprintln!("IN PROCESS, USING DATA SLICE {:?}", audio_data);

                // set distance, azimuth, and elevation for each channel
                saf_raw::binauraliserNF_setSourceDist_m(self.h_bin, i as i32, metadata.range);
                saf_raw::binauraliser_setSourceAzi_deg(self.h_bin, i as i32, metadata.azmuth * RAD_TO_DEGREE);
                saf_raw::binauraliser_setSourceElev_deg(self.h_bin, i as i32, metadata.elevation * RAD_TO_DEGREE);
                saf_raw::binauraliser_setSourceGain(self.h_bin, i as i32, metadata.gain);
            }

            // initialize codec variables, whatever those are, RIGHT before process
            saf_raw::binauraliserNF_initCodec(self.h_bin);

            // call process() to convert to binaural audio
            saf_raw::binauraliserNF_process(
                self.h_bin,
                raw_input_ptrs.as_ptr(),  // N inputs x K samples
                raw_output_ptrs.as_ptr(), // N inputs x K samples
                num_channels as i32,                 // N inputs
                NUM_OUT_CHANNELS as i32,             // N outputs
                FRAME_SIZE as i32                   // K samples
            );
            eprintln!("after process call unsafe");

            // convert raw pointers updated by process() back to vectors
            output_vec_1 = slice::from_raw_parts(raw_output_ptrs[0], FRAME_SIZE).to_vec();
            output_vec_2 = slice::from_raw_parts(raw_output_ptrs[1], FRAME_SIZE).to_vec();

            eprintln!("AFTER PROCESS, GOT OUT VEC 1: {:?}", output_vec_1);
            eprintln!("AFTER PROCESS, GOT OUT VEC 2: {:?}", output_vec_2);
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
    use std::ptr::read;

    use super::*;

    // Test BinauraliserNF constructor
    // #[test]
    // fn test_create_binauraliser() {
    //     BinauraliserNF::new();
    // }


    // #[test]
    // test with real sound file!
    // fn test_one_frame_sound() {
    //     eprintln!("in the test");

    //     let mut reader = WavReader::open(FILE_PATH).unwrap();

    //     let mut binauraliser_nf: BinauraliserNF = BinauraliserNF::new();
    //     eprintln!("before wav");

    //     let sample_vec = reader.samples::<i16>().step_by(2).map(|x| x.unwrap() as f32).collect::<Vec<_>>();

    //     const NUM_CHANNELS: usize = 1;
    //     eprintln!("NUM SAMPLES IS {}", FRAME_SIZE);

    //     let mock_meta_data: BufferMetadata = BufferMetadata {
    //         azmuth: 0.0,
    //         elevation: 0.0,
    //         range: 1.0,
    //         gain: 10.0
    //     };

    //     let spec = WavSpec{channels:2, sample_rate:44100, bits_per_sample:16, sample_format: SampleFormat::Int};
    //     let mut writer = WavWriter::create(OUT_FILE_PATH, spec).unwrap();

    //     // length is 
    //     let mock_buf_slice = &sample_vec[0..FRAME_SIZE];
    //     assert_eq!(mock_buf_slice.len(), FRAME_SIZE);

    //     let frame_slice = [(mock_meta_data, mock_buf_slice); NUM_CHANNELS];

    //     eprintln!("PASSING IN DATA SLICE {:?}", mock_buf_slice);
    //     let (left_vec, right_vec) = binauraliser_nf.process(frame_slice.as_ref());

    //     // loop written data to output file
    //     for (s1, s2) in std::iter::zip(left_vec, right_vec) {
    //         writer.write_sample(s1 as i16).unwrap();
    //         writer.write_sample(s2 as i16).unwrap();
    //     }

    //     writer.finalize().unwrap();
    // }

#[test]
    fn test_full_with_sound() {
        eprintln!("in the test");

        let mut reader = WavReader::open(FILE_PATH).unwrap();

        let mut binauraliser_nf: BinauraliserNF = BinauraliserNF::new();
        eprintln!("before wav");

        let sample_vec = reader.samples::<i16>().step_by(2).map(|x| x.unwrap() as f32).collect::<Vec<_>>();

        const NUM_CHANNELS: usize = 1;
        let NUM_SAMPLES: usize = sample_vec.len();
        eprintln!("NUM SAMPLES IS {}", NUM_SAMPLES);

        let mock_meta_data: BufferMetadata = BufferMetadata {
            azmuth: 0.0,
            elevation: 0.0,
            range: 1.0,
            gain: 10.0
        };

        eprintln!("before mock buffers");

        // let mock_buffers = [(mock_meta_data, sample_vec.as_slice()); NUM_CHANNELS];
        // println!("{:?}", mock_buffers[0].1.iter().take(10000).clone().collect::<Vec<_>>());

        let spec = WavSpec{channels:2, sample_rate:44100, bits_per_sample:16, sample_format: SampleFormat::Int};
        let mut writer = WavWriter::create(OUT_FILE_PATH, spec).unwrap();
        
        // here, we loop!

        // TESTING NOTES: for a non-deterministic i, we eventually seg fault
        for i in (0..NUM_SAMPLES - FRAME_SIZE).step_by(FRAME_SIZE) {
            let mock_buf_lo = i;
            let mock_buf_hi = i + FRAME_SIZE;
            eprintln!("lo: {} hi: {}", mock_buf_lo, mock_buf_hi);

            let mock_buf_slice = &sample_vec[mock_buf_lo..mock_buf_hi];
            eprintln!("Made mock buf slice len {} for {}", mock_buf_slice.len(), i);

            let frame_slice = [(mock_meta_data, mock_buf_slice); NUM_CHANNELS];
            eprintln!("Made frame slice for {}", i);

            let (left_vec, right_vec) = binauraliser_nf.process(frame_slice.as_ref());
            eprintln!("Finished process for {}", i);

            for (s1, s2) in std::iter::zip(left_vec, right_vec) {
                writer.write_sample(s1 as i16).unwrap();
                writer.write_sample(s2 as i16).unwrap();
            }

        }

        writer.finalize().unwrap();
    }

}
