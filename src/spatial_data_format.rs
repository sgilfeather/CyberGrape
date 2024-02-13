//! TODO
//!
//!

use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fs::File,
    io::{Read, Write},
    path::Path,
};

/// TODO
///
///
#[derive(Debug, Clone, PartialEq)]
struct GrapeFile {
    header: GrapeFileHeader,
    samples: Vec<f32>,
}

/// TODO
///
///
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct GrapeFileHeader {
    n_streams: u64,
    sample_rate: u64,
    tags: Vec<GrapeTag>,
}

/// TODO
///
///
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
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

/// TODO
///
///
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
    /// TODO
    ///
    ///
    pub fn builder() -> GrapeFileBuilder {
        GrapeFileBuilder::new()
    }

    /// TODO
    ///
    ///
    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<(), GrapeFileError> {
        let mut handle = File::create(path).map_err(GrapeFileError::IoError)?;

        let h_str = ron::ser::to_string(&self.header).map_err(GrapeFileError::RonError)?;
        let h_buf = h_str.as_bytes();

        handle.write_all(h_buf).map_err(GrapeFileError::IoError)?;

        handle.write_all(&[0xFF]).map_err(GrapeFileError::IoError)?;

        let s_buf: Vec<u8> = self.samples.iter().flat_map(|f| f.to_be_bytes()).collect();

        handle.write_all(&s_buf).map_err(GrapeFileError::IoError)?;

        Ok(())
    }

    /// TODO
    ///
    ///
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, GrapeFileError> {
        let mut handle = File::open(path).map_err(GrapeFileError::IoError)?;

        let mut raw_text = Vec::new();
        handle
            .read_to_end(&mut raw_text)
            .map_err(GrapeFileError::IoError)?;

        let delim_idx = raw_text
            .iter()
            .position(|e| *e == 0xFF)
            .ok_or(GrapeFileError::NoDelimiter)?;

        let (header_buf, samples_buf) = raw_text.split_at(delim_idx);
        let (_, samples_buf) = samples_buf
            .split_first()
            .ok_or(GrapeFileError::NoDelimiter)?;

        let header = ron::de::from_bytes::<GrapeFileHeader>(header_buf)
            .map_err(GrapeFileError::RonSpannedError)?;

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

    /// TODO
    ///
    ///
    pub fn streams_native_sample_rate(&self) -> (u64, Vec<(GrapeTag, Vec<f32>)>) {
        let sample_vecs = self.get_raw_streams();

        let res_vecs = Self::attach_tags(&self.header.tags, sample_vecs);

        (self.header.sample_rate, res_vecs)
    }

    /// TODO
    ///
    ///
    pub fn streams_with_sample_rate(&self, sample_rate: u64) -> Vec<(GrapeTag, Vec<f32>)> {
        match sample_rate.cmp(&self.header.sample_rate) {
            Ordering::Equal => self.streams_native_sample_rate().1,
            Ordering::Less => self.streams_quantized(sample_rate),
            Ordering::Greater => self.streams_interpolated(sample_rate),
        }
    }

    /// TODO
    ///
    ///
    fn attach_tags(tags: &[GrapeTag], samples: Vec<Vec<f32>>) -> Vec<(GrapeTag, Vec<f32>)> {
        assert_eq!(tags.len(), samples.len());
        tags.iter().cloned().zip(samples).collect()
    }

    /// TODO
    ///
    ///
    fn get_raw_streams(&self) -> Vec<Vec<f32>> {
        let n_streams = self.header.n_streams as usize;
        if n_streams == 0 {
            return Vec::new();
        }
        let stream_len = self.samples.len() / n_streams;
        let mut sample_vecs = vec![Vec::with_capacity(stream_len); n_streams];

        for (i, &e) in self.samples.iter().enumerate() {
            let stream_idx = i % n_streams;
            sample_vecs[stream_idx].push(e);
        }
        sample_vecs
    }

    /// TODO
    ///
    ///
    fn streams_interpolated(&self, sample_rate: u64) -> Vec<(GrapeTag, Vec<f32>)> {
        debug_assert!(sample_rate > self.header.sample_rate);
        let samples_per_pt = sample_rate as usize / self.header.sample_rate as usize;
        let raw_streams = self.get_raw_streams();
        let interpolated_streams = raw_streams
            .into_iter()
            .map(|v| {
                v.windows(2)
                    .flat_map(|w| {
                        let step = (w[1] - w[0]) / samples_per_pt as f32;
                        (0..samples_per_pt)
                            .map(|i| w[0] + i as f32 * step)
                            .collect::<Vec<_>>()
                    })
                    .collect()
            })
            .collect();
        Self::attach_tags(&self.header.tags, interpolated_streams)
    }

    /// TODO
    ///
    ///
    fn streams_quantized(&self, sample_rate: u64) -> Vec<(GrapeTag, Vec<f32>)> {
        debug_assert!(sample_rate < self.header.sample_rate);
        let pts_per_sample = self.header.sample_rate as usize / sample_rate as usize;
        let raw_streams = self.get_raw_streams();
        let quantized_streams = raw_streams
            .into_iter()
            .map(|v| {
                v.chunks(pts_per_sample)
                    .map(|c| c.iter().sum::<f32>() / c.len() as f32)
                    .collect()
            })
            .collect();
        Self::attach_tags(&self.header.tags, quantized_streams)
    }
}

/// TODO
///
///
#[derive(Debug, Clone)]
struct GrapeFileBuilder {
    n_streams: u64,
    sample_rate: u64,
    streams: Vec<(GrapeTag, Vec<f32>)>,
}

impl Default for GrapeFileBuilder {
    /// TODO
    ///
    ///
    fn default() -> Self {
        Self::new()
    }
}

impl GrapeFileBuilder {
    /// TODO
    ///
    ///
    fn new() -> Self {
        GrapeFileBuilder {
            n_streams: 0,
            sample_rate: 1000,
            streams: Vec::new(),
        }
    }

    /// TODO
    ///
    ///
    pub fn set_samplerate(self, sample_rate: u64) -> Self {
        GrapeFileBuilder {
            sample_rate,
            ..self
        }
    }

    /// TODO
    ///
    ///
    pub fn add_stream(mut self, stream: &[f32], tag: GrapeTag) -> Self {
        let stream: Vec<f32> = stream.to_vec();
        self.streams.push((tag, stream));
        self.n_streams += 1;
        self
    }

    /// TODO
    ///
    ///
    pub fn clear_streams(mut self) -> Self {
        self.streams.clear();
        self.n_streams = 0;
        self
    }

    /// TODO
    ///
    ///
    pub fn build_truncate(self) -> GrapeFile {
        let tags: Vec<GrapeTag> = self
            .streams
            .iter()
            .map(|(tag, _vec)| tag)
            .cloned()
            .collect();
        let sample_vecs: Vec<Vec<f32>> = self.streams.into_iter().map(|(_tag, vec)| vec).collect();

        let mut samples = Vec::new();

        if !sample_vecs.is_empty() {
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

    /// TODO
    ///
    ///
    pub fn build_extend(self) -> GrapeFile {
        let tags: Vec<GrapeTag> = self
            .streams
            .iter()
            .map(|(tag, _vec)| tag)
            .cloned()
            .collect();
        let sample_vecs: Vec<Vec<f32>> = self.streams.into_iter().map(|(_tag, vec)| vec).collect();

        let mut samples = Vec::new();

        if !sample_vecs.is_empty() {
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
                        *sample_vecs[stream_idx]
                            .get(sample_idx)
                            .unwrap_or(&lasts[stream_idx]),
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

    /// TODO
    ///
    ///
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
const A_FLOAT: f32 = 12.07843112945556640625;
mod tests {
    use super::*;

    #[test]
    fn write_and_read() {
        let tempfile = tempfile::NamedTempFile::new().unwrap();
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

    #[test]
    fn native_sample_rate_read() {
        let stream_data = vec![
            (GrapeTag::X, vec![A_FLOAT; 4]),
            (GrapeTag::Y, vec![A_FLOAT; 4]),
        ];

        let data = GrapeFile::builder()
            .set_samplerate(1000)
            .add_stream(&stream_data[0].1, stream_data[0].0)
            .add_stream(&stream_data[1].1, stream_data[1].0)
            .build()
            .unwrap();

        let (sr, streams) = data.streams_native_sample_rate();
        assert_eq!(1000, sr);
        assert_eq!(stream_data, streams);
    }

    #[test]
    fn quantize_read() {
        let stream_data = vec![
            (GrapeTag::X, vec![A_FLOAT; 4]),
            (GrapeTag::Y, vec![A_FLOAT; 4]),
        ];

        let data = GrapeFile::builder()
            .set_samplerate(1000)
            .add_stream(&stream_data[0].1, stream_data[0].0)
            .add_stream(&stream_data[1].1, stream_data[1].0)
            .build()
            .unwrap();

        let streams = data.streams_with_sample_rate(500);
        assert_eq!(
            vec![
                (GrapeTag::X, vec![A_FLOAT; 2]),
                (GrapeTag::Y, vec![A_FLOAT; 2]),
            ],
            streams
        );
    }

    #[test]
    fn interpolate_same() {
        let stream_data = vec![
            (GrapeTag::X, vec![A_FLOAT; 4]),
            (GrapeTag::Y, vec![A_FLOAT; 4]),
        ];

        let data = GrapeFile::builder()
            .set_samplerate(1000)
            .add_stream(&stream_data[0].1, stream_data[0].0)
            .add_stream(&stream_data[1].1, stream_data[1].0)
            .build()
            .unwrap();

        let streams = data.streams_with_sample_rate(2000);
        assert_eq!(
            vec![
                (GrapeTag::X, vec![A_FLOAT; 6]),
                (GrapeTag::Y, vec![A_FLOAT; 6]),
            ],
            streams
        );
    }

    #[test]
    fn interpolate_range() {
        let stream_data = vec![0.0, 0.2, 0.8];

        let data = GrapeFile::builder()
            .set_samplerate(5)
            .add_stream(&stream_data, GrapeTag::Yaw)
            .build()
            .unwrap();

        let streams = data.streams_with_sample_rate(10);
        assert_eq!(vec![(GrapeTag::Yaw, vec![0.0, 0.1, 0.2, 0.5])], streams);
    }

    #[test]
    fn read_from_empty() {
        let data = GrapeFile::builder().build().unwrap();

        let expected: Vec<(GrapeTag, Vec<f32>)> = vec![];
        let (_, streams1) = data.streams_native_sample_rate();
        let streams2 = data.streams_with_sample_rate(10);
        let streams3 = data.streams_with_sample_rate(10000);

        assert_eq!(expected, streams1);
        assert_eq!(expected, streams2);
        assert_eq!(expected, streams3);
    }
}
