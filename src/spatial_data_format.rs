//! TODO
//!
//!

#![allow(unused)]
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

const A_FLOAT: f32 = 12.07843112945556640625;

#[derive(Debug, Clone, PartialEq)]
struct GrapeFile {
    header: GrapeFileHeader,
    samples: Vec<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct GrapeFileHeader {
    n_streams: u64,
    sample_rate: u64,
    tags: Vec<GrapeTag>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
enum GrapeTag {
    X,
    Y,
    Z,
    Azimuth,
    Elevation,
    Range,
    Pitch,
    Yaw,
    Roll,
}

#[derive(Debug)]
enum GrapeFileError {
    UnequalSampleBufferLengths,
    NoDelimiter,
    TryInto,
    IoError(std::io::Error),
    RonError(ron::Error),
    RonSpannedError(ron::de::SpannedError),
}

impl GrapeFile {
    pub fn builder() -> GrapeFileBuilder {
        GrapeFileBuilder::new()
    }

    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<(), GrapeFileError> {
        let mut handle = File::create(path).map_err(|e| GrapeFileError::IoError(e))?;

        let h_str = ron::ser::to_string(&self.header).map_err(|e| GrapeFileError::RonError(e))?;
        let h_buf = h_str.as_bytes();

        handle
            .write_all(h_buf)
            .map_err(|e| GrapeFileError::IoError(e))?;

        handle
            .write_all(&[0xFF])
            .map_err(|e| GrapeFileError::IoError(e))?;

        let s_buf: Vec<u8> = self.samples.iter().flat_map(|f| f.to_be_bytes()).collect();

        handle
            .write_all(&s_buf)
            .map_err(|e| GrapeFileError::IoError(e))?;

        Ok(())
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, GrapeFileError> {
        let mut handle = File::open(path).map_err(|e| GrapeFileError::IoError(e))?;

        let mut raw_text = Vec::new();
        handle
            .read_to_end(&mut raw_text)
            .map_err(|e| GrapeFileError::IoError(e))?;

        let delim_idx = raw_text
            .iter()
            .position(|e| *e == 0xFF)
            .ok_or(GrapeFileError::NoDelimiter)?;

        let (header_buf, samples_buf) = raw_text.split_at(delim_idx);
        let (_, samples_buf) = samples_buf
            .split_first()
            .ok_or(GrapeFileError::NoDelimiter)?;

        let header = ron::de::from_bytes::<GrapeFileHeader>(header_buf)
            .map_err(|e| GrapeFileError::RonSpannedError(e))?;

        let samples: Vec<f32> = samples_buf
            .chunks(4)
            .map(|bs| -> Result<f32, GrapeFileError> {
                let four_bytes: [u8; 4] =
                    bs[0..4].try_into().map_err(|_| GrapeFileError::TryInto)?;
                Ok(f32::from_be_bytes(four_bytes))
            })
            .collect::<Result<Vec<f32>, GrapeFileError>>()?;

        Ok(GrapeFile { header, samples })
    }
}

#[derive(Debug, Clone)]
struct GrapeFileBuilder {
    n_streams: u64,
    sample_rate: u64,
    streams: Vec<(GrapeTag, Vec<f32>)>,
}

impl GrapeFileBuilder {
    fn new() -> Self {
        GrapeFileBuilder {
            n_streams: 0,
            sample_rate: 0,
            streams: Vec::new(),
        }
    }

    pub fn set_samplerate(self, sample_rate: u64) -> Self {
        GrapeFileBuilder {
            sample_rate,
            ..self
        }
    }

    pub fn add_stream(mut self, stream: &[f32], tag: GrapeTag) -> Self {
        let stream: Vec<f32> = stream.to_vec();
        self.streams.push((tag, stream));
        self.n_streams += 1;
        self
    }

    pub fn clear_streams(mut self) -> Self {
        self.streams.clear();
        self.n_streams = 0;
        self
    }

    pub fn build_truncate(self) -> GrapeFile {
        let tags: Vec<GrapeTag> = self
            .streams
            .iter()
            .map(|(tag, _vec)| tag)
            .cloned()
            .collect();
        let sample_vecs: Vec<Vec<f32>> = self.streams.into_iter().map(|(_tag, vec)| vec).collect();

        let mut samples = Vec::new();

        if sample_vecs.len() != 0 {
            let shortest = sample_vecs.iter().map(|v| v.len()).min().unwrap();
            samples.reserve_exact(shortest * self.n_streams as usize);
            for sample_idx in 0..shortest {
                for stream_idx in 0..self.n_streams {
                    samples.push(sample_vecs[stream_idx as usize][sample_idx]);
                }
            }
        };

        GrapeFile {
            header: GrapeFileHeader {
                n_streams: self.n_streams,
                sample_rate: self.sample_rate,
                tags,
            },
            samples,
        }
    }

    pub fn build_extend(self) -> GrapeFile {
        let tags: Vec<GrapeTag> = self
            .streams
            .iter()
            .map(|(tag, _vec)| tag)
            .cloned()
            .collect();
        let sample_vecs: Vec<Vec<f32>> = self.streams.into_iter().map(|(_tag, vec)| vec).collect();

        let mut samples = Vec::new();

        if sample_vecs.len() != 0 {
            let longest = sample_vecs.iter().map(|v| v.len()).max().unwrap();
            let lasts: Vec<f32> = sample_vecs
                .iter()
                .map(|v| v.last().unwrap_or(&0.0))
                .cloned()
                .collect();
            samples.reserve_exact(longest * self.n_streams as usize);
            for sample_idx in 0..longest {
                for stream_idx in 0..self.n_streams as usize {
                    samples.push(
                        sample_vecs[stream_idx as usize]
                            .get(sample_idx)
                            .unwrap_or(&lasts[stream_idx])
                            .clone(),
                    );
                }
            }
        };

        GrapeFile {
            header: GrapeFileHeader {
                n_streams: self.n_streams,
                sample_rate: self.sample_rate,
                tags,
            },
            samples,
        }
    }

    pub fn build(self) -> Result<GrapeFile, GrapeFileError> {
        let lens: Vec<usize> = self.streams.iter().map(|(_tag, v)| v.len()).collect();

        if lens.windows(2).all(|w| w[0] == w[1]) {
            Ok(self.build_truncate())
        } else {
            Err(GrapeFileError::UnequalSampleBufferLengths)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_and_read() {
        let mut tempfile = tempfile::NamedTempFile::new().unwrap();
        let path = tempfile.path();
        let data = GrapeFile::builder()
            .set_samplerate(1000)
            .add_stream(&vec![A_FLOAT; 4], GrapeTag::X)
            .add_stream(&vec![A_FLOAT; 4], GrapeTag::Y)
            .build()
            .unwrap();

        data.to_file(path).unwrap();
        let read_data = GrapeFile::from_file(path).unwrap();
        assert_eq!(data, read_data);
    }
}
