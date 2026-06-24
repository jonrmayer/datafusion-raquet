mod buffer;
mod error;
mod types;

pub use buffer::*;
pub use types::*;

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_get_pixel() {
//         let mut buffer = RasterBuffer::zeros(100, 100, RasterDataType::UInt8);
//         let _ = buffer.set_pixel(50, 50, 255.0);
//         let value = buffer.get_pixel(50, 50);
//         println!(" value {:?}", value);
//     }
//     #[test]
//     fn test_get_pixel_uint8() {
//         let raw_uint8 =
//             b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\x10".to_vec();
//         let buffer = RasterBuffer::new(raw_uint8, 4, 4, RasterDataType::UInt8, NoDataValue::None);
//         let value = buffer.unwrap().get_pixel(3, 3);
//         println!(" value {:?}", value);
//     }

//       #[test]
//     fn test_get_pixel_int16() {
//        let raw_int16 =b"\x64\x00\xC8\x00\x9C\xFF\x38\xFF".to_vec();
//         let buffer = RasterBuffer::new(raw_int16, 2, 2, RasterDataType::Int16, NoDataValue::None);

       
//         let value = buffer.unwrap().get_pixel(1, 1);
//         println!(" value {:?}", value);
//     }

    
// }
