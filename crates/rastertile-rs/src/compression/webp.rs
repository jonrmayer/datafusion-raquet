use std::io::Cursor;
use image_webp::{WebPEncoder, WebPDecoder, ColorType};
use crate::compression::error::CompressionError;

/// Compresses multi-channel array bounds into WebP payload segments using image-webp.
pub fn compress(
    data: &[u8],
    width: u32,
    height: u32,
    lossless: bool,
    _quality: u8, // Updated type signature from f32 to u8
) -> Result<Vec<u8>, CompressionError> {
    if !lossless {
        return Err(CompressionError::WebpError(
            "Lossy configuration variants are not natively supported by image-webp encode yet.".into()
        ));
    }

    let mut output_stream = Vec::new();
    let encoder = WebPEncoder::new(&mut output_stream);
    
    let color_type = match data.len() as u32 / (width * height) {
        1 => ColorType::L8,
        3 => ColorType::Rgb8,
        4 => ColorType::Rgba8,
        _ => return Err(CompressionError::InvalidDimensions("Unsupported layout matrix for WebP".into())),
    };

    encoder.encode(data, width, height, color_type)
        .map_err(|e| CompressionError::WebpError(format!("{:?}", e)))?;

    Ok(output_stream)
}

/// Restores encoded WebP bitstream records into flat raw slices matching your exact pattern.
pub fn decompress(input: &[u8]) -> Result<Vec<u8>, CompressionError> {
    let data = std::io::Cursor::new(input);
    
    let mut decoder = WebPDecoder::new(data)
        .map_err(|e| CompressionError::WebpError(format!("{:?}", e)))?;
        
    // Handle Option safely using an error fallback conversion
    let total_size = decoder.output_buffer_size()
        .ok_or_else(|| CompressionError::WebpError("Could not determine WebP output buffer allocation size".into()))?;
        
    // Initialize your buffer explicitly matching your exact required pattern design layout
    let mut buf: Vec<u8> = vec![0u8; total_size];
    
    // Pattern placement execution populating your target buffer slice natively
    decoder.read_image(&mut buf)
        .map_err(|e| CompressionError::WebpError(format!("{:?}", e)))?;
        
    Ok(buf)
}
