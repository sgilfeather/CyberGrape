pub struct TDBufMeta {
    data: Vec<Vec<BufferMetadata>>,
    num_tags: usize,
}

impl TDBufMeta {
    fn new(num_tags: usize) -> Self {
        Self {
            data: Vec::new(),
            num_tags,
        }
    }

    fn add(&mut self, data: &[BufferMetadata]) {
        assert_eq!(data.len(), self.num_tags);
        self.data.push(data.to_vec());
    }

    fn dump(&self) -> Vec<Vec<BufferMetadata>> {
        self.data
    }
}
