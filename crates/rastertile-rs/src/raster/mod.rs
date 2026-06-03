use crate::{get_pixel_value, parse_dtype};

pub fn decode_pixel(
    band_data: Vec<u8>,
    band_size: usize,
    dtype_str: &str,
    pixel_x: i32,
    pixel_y: i32,
    width: i32,
    compressed: bool,
) -> f64 {
    let dtype = parse_dtype(dtype_str);

    if pixel_x < 0 || pixel_y < 0 || width <= 0 {}

    let data = band_data;
    let data_size = band_size;

    let offset = (pixel_y) * width + pixel_x;
    let pixel_value = get_pixel_value(data, data_size, offset as usize, dtype);
    pixel_value
}

// double decode_pixel(const uint8_t *band_data, size_t band_size,
//                     const std::string &dtype_str,
//                     int pixel_x, int pixel_y, int width,
//                     bool compressed) {
//     BandDataType dtype = parse_dtype(dtype_str);

//     if (pixel_x < 0 || pixel_y < 0 || width <= 0) {
//         throw std::out_of_range("Invalid pixel coordinates or width");
//     }

//     const uint8_t *data;
//     size_t data_size;
//     std::vector<uint8_t> decompressed;

//     if (compressed) {
//         decompressed = decompress_gzip(band_data, band_size);
//         data = decompressed.data();
//         data_size = decompressed.size();
//     } else {
//         data = band_data;
//         data_size = band_size;
//     }

//     // Row-major order: offset = y * width + x
//     size_t offset = static_cast<size_t>(pixel_y) * width + pixel_x;
