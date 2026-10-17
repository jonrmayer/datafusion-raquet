use std::io::{Read, Write};
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use crate::compression::error::CompressionError;

/// Compresses raw byte sequences using standard Gzip compression profiles.
pub fn compress(data: &[u8], level: u32) -> Result<Vec<u8>, CompressionError> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
    encoder.write_all(data)?;
    let compressed_payload = encoder.finish()?;
    Ok(compressed_payload)
}

/// Inflates Gzip payload blocks cleanly back into uniform buffers.
pub fn decompress(compressed_data: &[u8]) -> Result<Vec<u8>, CompressionError> {
    let mut decoder = GzDecoder::new(compressed_data);
    let mut decompressed_payload = Vec::new();
    decoder.read_to_end(&mut decompressed_payload)
        .map_err(|e| CompressionError::GzipError(e.to_string()))?;
    Ok(decompressed_payload)
}
