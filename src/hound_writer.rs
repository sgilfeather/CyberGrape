//! A wrapper for the hound library that writes binauralized audio to
//! the user-speciifed output file.

use crate::component::{Component, ComponentError};
use hound::{Error as HoundError, WavSpec, WavWriter};

use std::fs::File;
use std::io::BufWriter;


/// A monitor wrapper for the hound WavWriter that writes out binauralized
/// audio
pub struct HoundWriter {
    writer: Option<WavWriter<BufWriter<File>>>,
}

impl HoundWriter {
    /// Instantiates a new HoundWriter, which wraps the hound WavWriter
    fn new(file: &'static str, wave_spec: WavSpec) -> Self {
        let writer = WavWriter::create(file, wave_spec).unwrap();

        Self {
            writer: Some(writer),
        }
    }
}

impl Component for HoundWriter {
    type InData = (Vec<f32>, Vec<f32>);
    type OutData = Result<(), HoundError>;

    /// Appends binauralized audio data to the specified output WAV file
    fn convert(self: &mut Self, input: (Vec<f32>, Vec<f32>)) -> Result<(), HoundError> {
        let (left_samps, right_samps) = input;
        let mut writer = self.writer.take().unwrap();

        // interleave the two streams and write the samples to the WAV file
        for (left, right) in std::iter::zip(left_samps, right_samps) {
            writer.write_sample(left).unwrap();
            writer.write_sample(right).unwrap();
        }

        // flush after each write to save state of the WAV file in the header
        let result = writer.flush();
        self.writer = Some(writer);
        return result;
    }

    /// Clean up WavWriter after writing all audio data from pipeline. This
    /// This happens automatically when the WavWriter is dropped, but
    /// calling this gives us controlled error checking.
    fn finalize(self: &mut Self) -> Result<(), ComponentError> {
        let writer = self.writer.take().unwrap();

        match writer.finalize() {
            hound::Result::Ok(()) => return Ok(()),
            hound::Result::Err(hound_error) => {
                return Err(ComponentError::HoundError(hound_error));
            }
        }
    }
}

impl ToString for HoundWriter {
    /// Converts the HoundWriter to a String, i.e. returns its name
    fn to_string(&self) -> String {
        "HoundWriter".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::run_component;
    use hound::{SampleFormat, WavReader};

    use std::f32::consts::PI;
    use std::fs::remove_file;
    use std::{io::BufReader, sync::mpsc::channel};

    const SAMP_RATE: u32 = 44100;
    const BITS_PER_SAMPLE: u16 = 32;
    const FRAME_SIZE: i32 = 128;

    const C: f32 = 261.61;

    fn create_sine_wave(frames: i32, note: f32) -> Vec<f32> {
        (0..frames)
            .map(|x| (x % FRAME_SIZE) as f32 / FRAME_SIZE as f32)
            .map(|t| (t * note * 2.0 * PI).sin() * (i16::MAX as f32))
            .collect()
    }

    // Write 100 sine wav frames to an output file using a WavWriter, and
    // read it back properly using a WavReader
    #[test]
    fn test_wav_writer_reader() {
        let file_name = "/Users/Skylar.Gilfeather/CyberGrape/sine.wav";

        let left_samps: Vec<f32> = create_sine_wave(100, C);
        let right_samps: Vec<f32> = create_sine_wave(100, C);

        // Create WavWriter and write 2 channel sine wav to output file
        let spec = WavSpec {
            channels: 2,
            sample_rate: SAMP_RATE,
            bits_per_sample: BITS_PER_SAMPLE,
            sample_format: SampleFormat::Float,
        };

        let mut writer = WavWriter::create(file_name, spec).unwrap();

        for (&left, &right) in std::iter::zip(&left_samps, &right_samps) {
            writer.write_sample(left).unwrap();
            writer.write_sample(right).unwrap();
        }

        assert!(writer.finalize().is_ok());

        // Create WavReader and read interleaved data back into 2 channel data
        let mut reader = WavReader::open(file_name).unwrap();

        let all_samps = reader
            .samples::<f32>()
            .collect::<Result<Vec<f32>, hound::Error>>()
            .unwrap();

        let left_samps_out = all_samps
            .clone()
            .into_iter()
            .step_by(2)
            .collect::<Vec<f32>>();
        let right_samps_out = all_samps
            .into_iter()
            .skip(1)
            .step_by(2)
            .collect::<Vec<f32>>();

        assert_eq!(left_samps, left_samps_out);
        assert_eq!(right_samps, right_samps_out);

        assert!(remove_file(file_name).is_ok());
    }

    /// Write 100 sine wav frames to an output file using a HoundWriter
    /// running as a Component thread. Then, read the data back using a
    /// WavReader and assert that all values are the same.
    #[test]
    fn test_hound_writer_read_write_one_frame() {
        let file_name = "/Users/Skylar.Gilfeather/CyberGrape/sine.wav";

        let left_samps: Vec<f32> = create_sine_wave(100, C);
        let right_samps: Vec<f32> = create_sine_wave(100, C);

        let hound_spec: WavSpec = WavSpec {
            channels: 2,
            sample_rate: SAMP_RATE,
            bits_per_sample: BITS_PER_SAMPLE,
            sample_format: SampleFormat::Float,
        };
        // Create HoundWriter and spawn as new Component thread
        let hound_writer = HoundWriter::new(file_name, hound_spec);

        let (hound_tx, hound_rx) = channel::<(Vec<f32>, Vec<f32>)>();
        let (result_tx, result_rx) = channel::<Result<(), HoundError>>();

        run_component(Box::new(hound_writer), hound_rx, result_tx);

        // Write single frame of sine wave data to the HoundWriter
        assert!(hound_tx
            .send((left_samps.clone(), right_samps.clone()))
            .is_ok());

        // Before creating reader, confirm that HoundWriter is done
        assert!(result_rx.recv().is_ok());

        // Read data back from file using WavReader
        let mut hound_reader: WavReader<BufReader<File>> = WavReader::open(file_name).unwrap();

        let all_samps = hound_reader
            .samples::<f32>()
            .collect::<Result<Vec<f32>, hound::Error>>()
            .unwrap();

        let left_samps_out = all_samps
            .clone()
            .into_iter()
            .step_by(2)
            .collect::<Vec<f32>>();
        let right_samps_out = all_samps
            .into_iter()
            .skip(1)
            .step_by(2)
            .collect::<Vec<f32>>();

        assert_eq!(left_samps, left_samps_out);
        assert_eq!(right_samps, right_samps_out);

        assert!(remove_file(file_name).is_ok());
    }
}
