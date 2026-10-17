mod error;

mod gzip;

mod jpeg;

mod webp;

use crate::CompressionFormat;

pub use error::CompressionError;

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
    pub fn decompress(&self, input: &[u8]) -> Result<Vec<u8>,CompressionError> {
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


// =====================================================================
// COMPRESSION PATH Configuration Structs (Updated with Integer Quality)
// =====================================================================

/// Configuration options for Gzip execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GzipConfig {
    pub level: u32,
}

/// Configuration options for JPEG spatial compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JpegConfig {
    pub width: u32,
    pub height: u32,
    pub quality: u8, // Already an integer type
}

/// Configuration options for WebP format encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebpConfig {
    pub width: u32,
    pub height: u32,
    pub lossless: bool,
    pub quality: u8, // Fixed: Changed from f32 to u8 to allow safe `Eq` derivation
}

/// Strongly-typed configuration rules instantiated and triggered explicitly by your pipeline code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressConfiguration {
    None,
    Gzip(GzipConfig),
    Jpeg(JpegConfig),
    Webp(WebpConfig),
}

/// Central routing controller processing raw byte buffers down to target formats via code configs.
pub fn compress(data: &[u8], config: &CompressConfiguration) -> Result<Vec<u8>, CompressionError> {
    match config {
        CompressConfiguration::None => Ok(data.to_vec()),
        CompressConfiguration::Gzip(cfg) => gzip::compress(data, cfg.level),
        CompressConfiguration::Jpeg(cfg) => jpeg::compress(data, cfg.width, cfg.height, cfg.quality),
        CompressConfiguration::Webp(cfg) => webp::compress(data, cfg.width, cfg.height, cfg.lossless, cfg.quality),
    }
}
