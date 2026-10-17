use std::io::Cursor;
use crate::compression::error::CompressionError;

/// Compresses explicit image coordinate bounds directly into visual JPEG segments.
pub fn compress(
    data: &[u8], 
    width: u32, 
    height: u32, 
    quality: u8
) -> Result<Vec<u8>, CompressionError> {
    let mut output_stream = Vec::new();
    let mut encoder = jpeg_encoder::Encoder::new(&mut output_stream, quality);
    
    let color_type = match data.len() as u32 / (width * height) {
        1 => jpeg_encoder::ColorType::Luma,
        3 => jpeg_encoder::ColorType::Rgb,
        _ => return Err(CompressionError::InvalidDimensions("Invalid channels for JPEG formatting".into())),
    };

    encoder.encode(data, width as u16, height as u16, color_type)
        .map_err(|e| CompressionError::JpegError(e.to_string()))?;
        
    Ok(output_stream)
}

/// Unpacks operational lossy JPEG bitstreams into multi-band channel sequences.
pub fn decompress(compressed_data: &[u8]) -> Result<Vec<u8>, CompressionError> {
    let stream_cursor = Cursor::new(compressed_data);
    let mut decoder = jpeg_decoder::Decoder::new(stream_cursor);
    
    decoder.decode()
        .map_err(|e| CompressionError::JpegError(e.to_string()))
}
