mod error;

mod gzip;

mod jpeg;

mod webp;

use crate::CompressionFormat;

pub use crate::compression::error::{CompressionError, CompressionResult};

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
    pub fn decompress(&self, input: &[u8]) -> CompressionResult<Vec<u8>> {
        if input.is_empty() {
            return Ok(Vec::new());
        }

        match self.format {
            CompressionFormat::None => Ok(input.to_vec()),
            CompressionFormat::Gzip => gzip::decompress(input),
            CompressionFormat::Jpeg => jpeg::decompress(input),
            CompressionFormat::WebP => webp::decompress(input),
        }
    }
}
