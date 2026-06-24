use image_webp::WebPDecoder;
use crate::compression::error::CompressionResult;

pub fn decompress(input: &[u8]) -> CompressionResult<Vec<u8>> {
    let data = std::io::Cursor::new(input);
    let mut decoder = WebPDecoder::new(data).unwrap();
    let mut buf: Vec<u8> = Vec::new();
    decoder.read_image(&mut buf)?;
    Ok(buf)
}
