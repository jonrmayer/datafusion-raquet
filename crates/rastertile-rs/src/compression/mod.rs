mod error;

mod gzip;

mod jpeg;

mod webp;

use crate::CompressionFormat;

pub struct Compression {
    pub format: CompressionFormat,
}

impl Compression {
    /// Create a new Compression codec with default format
    pub fn new() -> Self {
        Self {
            format: CompressionFormat::default(),
        }
    }
    /// Decompress  data
    pub fn decompress(&self, input: &[u8]) -> Vec<u8> {
        if input.is_empty() {
            return Vec::new();
        }

        match self.format {
            CompressionFormat::None => input.to_vec(),
            CompressionFormat::Gzip => gzip::decompress(input),
            CompressionFormat::Jpeg => jpeg::decompress(input),
            CompressionFormat::WebP => webp::decompress(input),
        }
    }
}
