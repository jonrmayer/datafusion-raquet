use crate::compression::error::CompressionResult;
use flate2::read::GzDecoder;
use std::io::prelude::*;

pub fn decompress(input: &[u8]) -> CompressionResult<Vec<u8>> {
    let mut d = GzDecoder::new(input);
    let mut buf: Vec<u8> = Vec::new();
    d.read_to_end(&mut buf)?;
    Ok(buf)
}
