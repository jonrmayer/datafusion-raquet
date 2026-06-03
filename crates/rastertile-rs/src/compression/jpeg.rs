use jpeg_decoder::Decoder;

pub fn decompress(input: &[u8]) -> Vec<u8> {
    let mut decoder = Decoder::new(input);
    let pixels = decoder.decode().unwrap();
    pixels
}
