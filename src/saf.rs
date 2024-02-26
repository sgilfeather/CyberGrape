//! A safe api into the Spatial Audio Framework.

use std::ptr::{addr_of_mut, null, null_mut};

use crate::saf_raw;

use libc::c_void;

// Sets all audio channel distances to 1 meter—— stretch goal to specify per channel
const DIST_DEFAULT: f32 = 1.0;
const SAMP_RATE: i32 = 44100;
const NUM_OUT_CHANNELS: usize = 2;
const FRAME_SIZE: usize = 128;

const RAD_TO_DEGREE: f32 = 180.0 / std::f32::consts::PI;

///
/// A Binauraliser is anything that can take an array of sound buffers, paired
/// with their associated metadata, and return a pair of freshly allocated
/// buffers representing the mixed stereo audio.
///
pub trait Binauraliser {
    fn new() -> Self;
    fn process_frame(&mut self, buffers: &[(BufferMetadata, &[f32])]) -> (Vec<f32>, Vec<f32>);

    ///
    /// Takes a slice of audio data tuples for each sound source. Each tuple
    /// contains float sound data and a BufferMetadata, which encodes the
    /// sound source's location, range, and gain over that frame period.
    ///
    fn process(&mut self, buffers: &[(BufferMetadata, &[f32])]) -> (Vec<f32>, Vec<f32>) {
        let num_samples = buffers
            .iter()
            .map(|(_tag, samples)| samples.len())
            .max()
            .unwrap_or(0);
        let mut final_left_vec = Vec::with_capacity(num_samples);
        let mut final_right_vec = Vec::with_capacity(num_samples);

        for i in (0..(num_samples - FRAME_SIZE)).step_by(FRAME_SIZE) {
            let buf_lo = i;
            let buf_hi = i + FRAME_SIZE;

            let frame = buffers
                .iter()
                .map(|(metadata, samples)| (*metadata, &samples[buf_lo..buf_hi]))
                .collect::<Vec<_>>();

            let (mut left_vec, mut right_vec) = self.process_frame(&frame);

            final_left_vec.append(&mut left_vec);
            final_right_vec.append(&mut right_vec);
        }
        (final_left_vec, final_right_vec)
    }
}

/// The metadata associated with an audio stream. Includes the buffer's angular
/// position, range, and gain.
#[derive(Clone, Copy)]
pub struct BufferMetadata {
    pub azimuth: f32,
    pub elevation: f32,
    pub range: f32,
    // gain: amount of amplification applied to a signal
    // ratio between the input volume and the output volume
    pub gain: f32,
}

/// Impl of Binauraliser that uses SAF's BinauraliserNF (Near Field)
pub struct BinauraliserNF {
    // stores C-style BinauraliserNF object, for use in libsaf
    h_bin: *mut c_void,
}

impl Binauraliser for BinauraliserNF {
    /// Creates a new BinauraliserNF
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

    ///
    /// Takes a slice of audio data tuples for each sound source. Each tuple
    /// contains 128 frames of float sound data and a BufferMetadata,
    /// which encodes the sound source's location, range, and gain over that
    /// frame period.
    ///
    /// Returns a pair of vectors containing the mixed binaural audio.
    ///
    /// Invariant: All input buffers must be the same length, of 128
    ///
    fn process_frame(&mut self, buffers: &[(BufferMetadata, &[f32])]) -> (Vec<f32>, Vec<f32>) {
        for (_, b) in buffers {
            debug_assert_eq!(b.len(), FRAME_SIZE);
        }
        // convert each slice in buffers to a raw pointer
        let num_channels: usize = buffers.len();

        // allocate input and output buffers for process() call
        let mut raw_input_ptrs: Vec<*const f32> = vec![null(); num_channels];

        let mut output_vec_1 = vec![0.0; FRAME_SIZE];
        let mut output_vec_2 = vec![0.0; FRAME_SIZE];

        let mut raw_output_ptrs: Vec<*mut f32> = vec![null_mut(); NUM_OUT_CHANNELS];

        raw_output_ptrs[0] = output_vec_1.as_mut_ptr();
        raw_output_ptrs[1] = output_vec_2.as_mut_ptr();

        unsafe {
            saf_raw::binauraliser_setNumSources(self.h_bin, num_channels as i32);

            for (i, &(metadata, audio_data)) in buffers.iter().enumerate() {
                // store raw pointer for channel in raw_data_ptrs
                raw_input_ptrs[i] = audio_data.as_ptr();

                // set distance, azimuth, and elevation for each channel
                saf_raw::binauraliserNF_setSourceDist_m(self.h_bin, i as i32, metadata.range);
                saf_raw::binauraliser_setSourceAzi_deg(
                    self.h_bin,
                    i as i32,
                    metadata.azimuth * RAD_TO_DEGREE,
                );
                saf_raw::binauraliser_setSourceElev_deg(
                    self.h_bin,
                    i as i32,
                    metadata.elevation * RAD_TO_DEGREE,
                );
                saf_raw::binauraliser_setSourceGain(self.h_bin, i as i32, metadata.gain);
            }

            // note: must initialize codec variables after setting positional
            // data for each of the sound sources
            saf_raw::binauraliserNF_initCodec(self.h_bin);

            // call process() to convert to binaural audio
            saf_raw::binauraliserNF_process(
                self.h_bin,
                raw_input_ptrs.as_ptr(),  // N inputs x K samples
                raw_output_ptrs.as_ptr(), // N inputs x K samples
                num_channels as i32,      // N inputs
                NUM_OUT_CHANNELS as i32,  // N outputs
                FRAME_SIZE as i32,        // K samples
            );
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

struct DummyBinauraliser {}

impl Binauraliser for DummyBinauraliser {
    fn new() -> Self {
        Self {}
    }
    fn process_frame(&mut self, buffers: &[(BufferMetadata, &[f32])]) -> (Vec<f32>, Vec<f32>) {
        for (_, b) in buffers {
            debug_assert_eq!(b.len(), FRAME_SIZE);
        }
        assert!(buffers.len() == 2);
        (buffers[0].1.to_vec(), buffers[1].1.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use hound::{SampleFormat, WavReader, WavSpec, WavWriter};
    use std::f32::consts::PI;

    const MOCK_METADATA: BufferMetadata = BufferMetadata {
        azimuth: 0.0,
        elevation: 0.0,
        range: 1.0,
        gain: 1.0,
    };

    const LEFT_METADATA: BufferMetadata = BufferMetadata {
        azimuth: 90.0,
        elevation: 10.0,
        range: 1.0,
        gain: 1.0,
    };

    const RIGHT_METADATA: BufferMetadata = BufferMetadata {
        azimuth: 90.0,
        elevation: 0.0,
        range: 1.0,
        gain: 1.0,
    };

    const C: f32 = 261.61;
    const G: f32 = 392.00;

    fn create_sine_wave(frames: i32, note: f32) -> Vec<f32> {
        (0..frames)
            .map(|x| (x % 44100) as f32 / 44100.0)
            .map(|t| (t * note * 2.0 * PI).sin() * (i16::MAX as f32))
            .collect()
    }

    fn write_stereo_output(left_samps: Vec<f32>, right_samps: Vec<f32>, out_file: &'static str) {
        let spec = WavSpec {
            channels: 2,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create(out_file, spec).unwrap();

        for (left, right) in std::iter::zip(left_samps, right_samps) {
            writer.write_sample(left as i16).unwrap();
            writer.write_sample(right as i16).unwrap();
        }

        writer.finalize().unwrap();
    }

    #[test]
    ///
    /// Validate that runnning process_frame() doesn't segfault on mono
    /// audio data
    ///
    fn test_mono_single_frame() {
        let mut binauraliser_nf = BinauraliserNF::new();

        // 1 frame of audio (128 samples)
        let c_note_vec: Vec<f32> = create_sine_wave(FRAME_SIZE as i32, C);
        let frame_slice = [(MOCK_METADATA, &c_note_vec[0..FRAME_SIZE])];

        // assert no segfault and that data is non-null
        let (left_samps, right_samps) = binauraliser_nf.process_frame(frame_slice.as_ref());
        assert!(left_samps.into_iter().all(|x| x != 0.0));
        assert!(right_samps.into_iter().all(|x| x != 0.0));
    }

    #[test]
    ///
    /// Validate that runnning process_frame() doesn't segfault on stereo
    /// audio data
    ///
    fn test_stereo_single_frame() {
        let mut binauraliser_nf = BinauraliserNF::new();

        // 1 frame of audio (128 samples)
        let c_note_vec: Vec<f32> = create_sine_wave(FRAME_SIZE as i32, C);
        let g_note_vec: Vec<f32> = create_sine_wave(FRAME_SIZE as i32, G);

        let frame_slice = [
            (LEFT_METADATA, &c_note_vec[0..FRAME_SIZE]),
            (RIGHT_METADATA, &g_note_vec[0..FRAME_SIZE]),
        ];

        // assert no segfault and that data is non-null
        let (left_samps, right_samps) = binauraliser_nf.process_frame(frame_slice.as_ref());
        assert!(left_samps.into_iter().all(|x| x != 0.0));
        assert!(right_samps.into_iter().all(|x| x != 0.0));
    }

    #[test]
    fn test_stereo_multi_frame() {
        let mut binauraliser_nf = BinauraliserNF::new();

        const THREE_SEC: i32 = SAMP_RATE * 3;
        // 1 frame of audio (128 samples)
        let c_note_vec: Vec<f32> = create_sine_wave(THREE_SEC, C);
        let g_note_vec: Vec<f32> = create_sine_wave(THREE_SEC, G);

        let frame_slice = [
            (LEFT_METADATA, &c_note_vec[0..THREE_SEC as usize]),
            (RIGHT_METADATA, &g_note_vec[0..THREE_SEC as usize]),
        ];

        // assert no segfault and that data is non-null
        let (left_samps, right_samps) = binauraliser_nf.process(frame_slice.as_ref());
        assert!(left_samps.clone().into_iter().all(|x| x != 0.0));
        assert!(right_samps.clone().into_iter().all(|x| x != 0.0));

        // toggle writing output
        // write_stereo_output(left_samps, right_samps, "/Users/Skylar.Gilfeather/Desktop/out.wav");
    }
}
