//! A wrapper for the hound library that writes binauralized audio to
//! the user-speciifed output file.

use crate::component::Component;
use hound::{Error, SampleFormat, WavReader, WavSpec, WavWriter};
use std::f32::consts::PI;
use std::fs::File;
use std::io::BufWriter;

const SAMP_RATE: u32 = 44100;
const NUM_OUT_CHANNELS: u32 = 2;
const BITS_PER_SAMPLE: u16 = 16;
const FRAME_SIZE: i32 = 128;

/// A monitor wrapper for the hound WavWriter that writes out binauralized
/// audio
pub struct HoundWriter {
    writer: Option<WavWriter<BufWriter<File>>>
}

impl HoundWriter {
    /// Instantiates a new HoundWriter, which wraps the hound WavWriter
    /// TODO: add new method to trait and enforce that params are passed in as struct in a box
    fn new(file: &'static str, channels: u16, sample_rate: u32, bits_per_sample: u16) -> Self {
        let spec = WavSpec {
            channels,
            sample_rate,
            bits_per_sample,
            sample_format: SampleFormat::Int,
        };
        let writer = WavWriter::create(file, spec).unwrap();

        Self { writer: Some(writer) }
    }
}

impl Component for HoundWriter {
    type InData = (Vec<f32>, Vec<f32>);
    type OutData = Result<(), Error>;

    /// Appends binauralized audio data to the specified output WAV file
    fn convert(self: &mut Self, input: (Vec<f32>, Vec<f32>)) -> Result<(), Error> {
        let (left_samps, right_samps) = input;
        let mut writer = self.writer.take().unwrap();

        // interleave the two streams and write the samples to the WAV file
        for (left, right) in std::iter::zip(left_samps, right_samps) {
            writer.write_sample(left as i16).unwrap();
            writer.write_sample(right as i16).unwrap();
        }

        // flush after each write to save state of the WAV file in the header
        let result = writer.flush();
        self.writer = Some(writer);
        return result;
    }

    /// Clean up WavWriter after writing all audio data from pipeline. This
    /// This happens automatically when the WavWriter is dropped, but
    /// calling this gives us controlled error checking.
    fn finalize(self: &mut Self) -> Result<(), String> {
        let writer = self.writer.take().unwrap();

        match writer.finalize() {
            Ok(()) => {
                return Ok(())
            },
            Err(hound_error) => {
                Err(format!("{hound_error:?}"))
            },
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
    use std::{io::BufReader, sync::mpsc::channel};

    const C: f32 = 261.61;

    fn create_sine_wave(frames: i32, note: f32) -> Vec<f32> {
        (0..frames)
            .map(|x| (x % FRAME_SIZE) as f32 / FRAME_SIZE as f32)
            .map(|t| (t * note * 2.0 * PI).sin() * (i16::MAX as f32))
            .collect()
    }

    /// Write a frame of a sine wav to an output file using a HoundWriter
    /// running as a Component thread. Then, read the data back using a
    /// WavReader and assert that all values are the same.
    #[test]
    fn test_read_write_audio() {
        let file_name = "/Users/Skylar.Gilfeather/CyberGrape/c.wav";

        let left_samps: Vec<f32> = create_sine_wave(1, C);
        let right_samps: Vec<f32> = create_sine_wave(1, C);

        // Create HoundWriter and spawn as new Component thread
        let hound_writer = HoundWriter::new(file_name, 2, SAMP_RATE, BITS_PER_SAMPLE);

        let (hound_tx, hound_rx) = channel::<(Vec<f32>, Vec<f32>)>();
        let (result_tx, _) = channel::<Result<(), Error>>();

        run_component(Box::new(hound_writer), hound_rx, result_tx);

        // Write single frame of sine wave data to the HoundWriter
        assert_eq!(hound_tx.send((left_samps.clone(), right_samps.clone())), 
                   Ok(()));

        // Read data back from file using WavReader
        let mut hound_reader: WavReader<BufReader<File>> = WavReader::open(file_name).unwrap();
    
        let all_samps = hound_reader
            .samples::<f32>()
            .collect::<Result<Vec<f32>, hound::Error>>()
            .unwrap();

        let left_samps_out = all_samps.clone().into_iter().step_by(2).collect::<Vec<f32>>();
        let right_samps_out = all_samps.into_iter().skip(1).step_by(2).collect::<Vec<f32>>();

        assert_eq!(left_samps, left_samps_out);
        assert_eq!(right_samps, right_samps_out);
    }
}
