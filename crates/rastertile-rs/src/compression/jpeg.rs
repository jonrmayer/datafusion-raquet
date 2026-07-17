use crate::compression::error::CompressionResult;
use jpeg_decoder::Decoder;
pub fn decompress(input: &[u8]) -> CompressionResult<Vec<u8>> {
    let mut decoder = Decoder::new(input);
    let pixels = decoder.decode()?;
    Ok(pixels)
}
