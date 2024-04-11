//! A wrapper for the hound library that is suited for CyberGrape's toolchain

use super::*;

use hound::{SampleFormat, WavReader, WavSpec, WavWriter};

const SAMP_RATE: i32 = 44100;
const NUM_OUT_CHANNELS: i32 = 2;
const BITS_PER_SAMPLE: i32 = 16;
const FRAME_SIZE: usize = 128;

// init handles creation of the WavWriter, using create(), which creates the 
// file and the writer for it, then returns the writer for use in 
// append_stereo_output. 
// 
// requires the name of the file as a string, and the return of this call must 
// be passed into append_stereo_output in addition to the sample vecs
fn init_stereo_output(out_file: &'static str) -> WavWriter {

    let spec = WavSpec {
        channels: NUM_OUT_CHANNELS,
        sample_rate: SAMP_RATE,
        bits_per_sample: BITS_PER_SAMPLE,
        sample_format: SampleFormat::Int,
    };

    WavWriter::create(out_file, spec).unwrap()
}

// can be called as many times as necessary in a loop to append audio data to 
// an existing WAV file
//
// must first call init_stereo_output to initialize the WavWriter, and pass in
// the resulting writer as an argument each time this is called
fn append_stereo_output(left_samps: Vec<f32>, right_samps: Vec<f32>, writer: WavWriter) {
    
    //interleave the two streams and write the samples to the WAV file
    for (left, right) in std::iter::zip(left_samps, right_samps) {
        writer.write_sample(left as i16).unwrap();
        writer.write_sample(right as i16).unwrap();
    }

    // flush after each write to save the state of the WAV file in the header
    writer.flush().unwrap();
}

// finalize once all samples have been written to the WAV file. this will also
// happen automatically once the WavWriter is dropped, but calling this allows 
// for controlled error checking 
fn finalize_stereo_output(writer:WavWriter) {
    writer.finalize().unwrap()
}

// open a WAV file and return a WavReader, which then has a host of useful 
// operations that are documented in the the hound specs:
//      https://docs.rs/hound/latest/hound/struct.WavReader.html
//
//      most notably:
//          .samples() / into_samples() which return iterators over samples
//          .duration() which returns duration of the file in samples
//          .len() which returns the number of values that samples() iterator 
//                                                          will yield
//
fn init_wav_input(in_file: &'static str) -> WavReader {
    let mut fp = File::open(in_file);
    WavReader::new(fp).unwrap()
}

// given a WavReader, destroys it and cleans up memory associated with the
// underlying File reader as well
fn destroy_wav_input(reader: WavReader) {
    let mut r = WavReader::into_inner();

    // ensure file is closed 
    drop(r); // this call ignores errors, so can remove this line if the file 
             // is guaranteed to go out of scope anyways in the client 
}