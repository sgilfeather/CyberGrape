//! Where we store our time-domain spatial data.

use crate::saf::BufferMetadata;

/// A buffer to store our time-domain spatial data. Ensures that we always
/// have data for each tag for each time slice.
#[derive(Debug, Clone)]
pub struct TDBufMeta {
    data: Vec<Vec<BufferMetadata>>,
    num_tags: usize,
}

impl TDBufMeta {
    /// Instantiate the buffer so that it ensures that there are `num_tags`
    /// metadata instances each time we call [`TDBufMeta::add`].
    pub fn new(num_tags: usize) -> Self {
        Self {
            data: Vec::new(),
            num_tags,
        }
    }

    /// Insert a time-slice's worth of metadata into the buffer. Panics if
    /// there is the wrong number of metadata entries.
    pub fn add(&mut self, data: Vec<BufferMetadata>) {
        assert_eq!(data.len(), self.num_tags);
        self.data.push(data);
    }

    /// Return all of the metadata that we have collected, consuming the buffer.
    pub fn dump(self) -> Vec<Vec<BufferMetadata>> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buf_init() {
        let buf = TDBufMeta::new(2);
        assert_eq!(buf.num_tags, 2);
        let update = [
            BufferMetadata {
                azimuth: 90.0,
                elevation: 0.0,
                range: 1.0,
                gain: 1.0,
            },
            BufferMetadata {
                azimuth: 45.0,
                elevation: 45.0,
                range: 1.0,
                gain: 1.0,
            },
        ];
        let mut buf = TDBufMeta::new(2);
        buf.add(update.to_vec());
        let data = buf.dump();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].len(), 2);
    }
}
