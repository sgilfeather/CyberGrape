use crate::saf::BufferMetadata;

#[derive(Debug, Clone)]
pub struct TDBufMeta {
    data: Vec<Vec<BufferMetadata>>,
    num_tags: usize,
}

impl TDBufMeta {
    pub fn new(num_tags: usize) -> Self {
        Self {
            data: Vec::new(),
            num_tags,
        }
    }

    pub fn add(&mut self, data: &[BufferMetadata]) {
        assert_eq!(data.len(), self.num_tags);
        self.data.push(data.to_vec());
    }

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
        buf.add(&update);
        let data = buf.dump();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].len(), 2);
    }
}
