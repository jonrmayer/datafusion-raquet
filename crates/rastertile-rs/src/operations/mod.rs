mod array;
// mod data_type;
mod error;

mod nd_array;

mod statistics;

mod utils;

mod decode;

mod encode;

use std::sync::Arc;

#[cfg(test)]
mod tests;

pub use utils::{decode_array, filter_float_array, get_pixel, no_data, no_filter_float_array};

pub use decode::{
    decode_native_array_f32, decode_native_array_f64, decode_native_array_i8,
    decode_native_array_i16, decode_native_array_i32, decode_native_array_i64,
    decode_native_array_u8, decode_native_array_u16, decode_native_array_u32,
    decode_native_array_u64,
};

pub use encode::encode_typed_array;
pub use encode::{
    encode_native_array_f32, encode_native_array_f64, encode_native_array_i8,
    encode_native_array_i16, encode_native_array_i32, encode_native_array_i64,
    encode_native_array_u8, encode_native_array_u16, encode_native_array_u32,
    encode_native_array_u64,
};

pub use statistics::TileStatistics;

pub use array::{Array, TypedArray};

// pub use data_type::DataType;
pub use error::{OperationsError, OperationsResult};

use crate::compression::{CompressConfiguration, compress as codec_compress};

use crate::Compression;

use crate::Metadata;

// use ndarray::Array3;

// use nd_array::NdArray;

use rolling_stats::Stats;

/// Helper trait mapping primitive Rust types directly to their corresponding TypedArray variant containers.
pub trait ToTypedArray: Sized {
    fn to_typed_array(data: Vec<Self>) -> TypedArray;
    fn to_typed_array_from_options(data: &[Option<Self>], fallback_no_data: Self) -> TypedArray;
}

macro_rules! impl_to_typed_array_trait {
    ($t:ty, $variant:ident) => {
        impl ToTypedArray for $t {
            fn to_typed_array(data: Vec<Self>) -> TypedArray {
                TypedArray::$variant(data)
            }

            fn to_typed_array_from_options(
                data: &[Option<Self>],
                fallback_no_data: Self,
            ) -> TypedArray {
                let flattened: Vec<Self> = data
                    .iter()
                    .map(|opt| opt.unwrap_or(fallback_no_data))
                    .collect();
                TypedArray::$variant(flattened)
            }
        }
    };
}

// Generate the complete suite of conversion implementations across the primitive type matrix
impl_to_typed_array_trait!(i8, Int8);
impl_to_typed_array_trait!(u8, UInt8);
impl_to_typed_array_trait!(i16, Int16);
impl_to_typed_array_trait!(u16, UInt16);
impl_to_typed_array_trait!(i32, Int32);
impl_to_typed_array_trait!(u32, UInt32);
impl_to_typed_array_trait!(i64, Int64);
impl_to_typed_array_trait!(u64, UInt64);
impl_to_typed_array_trait!(f32, Float32);
impl_to_typed_array_trait!(f64, Float64);

pub struct Operations {
    pub metadata: Metadata,
}
impl Operations {
    pub fn new(metadata: Metadata) -> Self {
        Self { metadata }
    }

    fn metadata(&self) -> Metadata {
        self.metadata.clone()
    }

    fn no_data(&self) -> Option<f64> {
        no_data(self.metadata().data_type(), self.metadata().no_data())
    }

    // /// Compresses a strongly-typed native array slice directly into a chosen binary compression format.
    // /// This routes your data through the exact encode pattern step before invoking the core compression engines.
    // pub fn compress<T>(
    //     &self,
    //     input: &[Option<T>],
    //     fallback_no_data: T,
    //     config: &CompressConfiguration,
    // ) -> OperationsResult<Vec<u8>>
    // where
    //     T: ToTypedArray,
    // {
    //     // 1. Leverage the updated encode function to extract the flat uncompressed raw bytes
    //     let uncompressed_bytes = self.encode(input, fallback_no_data)?;

    //     // 2. Dispatch the raw byte block down to the central compression codec router
    //     let compressed_payload = codec_compress(&uncompressed_bytes, config).map_err(|e| {
    //         OperationsError::General(format!("Compression engine failure: {:?}", e))
    //     })?;

    //     Ok(compressed_payload)
    // }

    pub fn decompress(&self, input: Option<&[u8]>) -> OperationsResult<Vec<u8>> {
        let compression = Compression {
            format: self.metadata().compression(),
        };
        let data = input.unwrap();
        let decompressed = compression.decompress(data)?;
        Ok(decompressed)
    }
    fn raw_array(&self, input: Option<&[u8]>) -> OperationsResult<Array> {
        let decompressed = self.decompress(input)?;

        let shape = [
            self.metadata().samples(),
            self.metadata().tile_size(),
            self.metadata().tile_size(),
        ];
        Array::try_new(decompressed, shape, Some(self.metadata().data_type()))
    }

    /// Converts a structured Array container back into its flat raw binary payload using zero-copy casting.
    pub fn raw_bytes(&self, array: Array) -> OperationsResult<Vec<u8>> {
        encode_typed_array(array.data())
    }

    //     fn convert_array(inner_array: Arc<dyn Array>) {
    //     }
    // // }

    //     pub fn encode(&self, inner_array: Array) -> OperationsResult<Vec<u8>> {

    //         let data = input.unwrap();
    //         let decompressed = compression.decompress(data)?;
    //         Ok(decompressed)
    //     }

    // pub fn raw_encode<T>(
    //     &self,
    //     input: &[Option<T>],
    //     fallback_no_data: T,
    // ) -> OperationsResult<Vec<u8>>
    // where
    //     T: ToTypedArray,
    // {
    //     // 1. Wrap the optional native primitives into the target TypedArray variant
    //     let typed_array = T::to_typed_array_from_options(input, fallback_no_data);

    //     // 2. Establish structural shape parameters matching target metadata requirements
    //     let shape = [
    //         self.metadata().samples() as usize,
    //         self.metadata().tile_size() as usize,
    //         self.metadata().tile_size() as usize,
    //     ];

    //     // 3. Instantiate the Array container directly by passing the typed_array token
    //     let raw_array = Array {
    //         data: typed_array,
    //         shape,
    //         data_type: Some(self.metadata().data_type()),
    //     };

    //     // 4. Extract raw uncompressed binary bytes via your zero-copy cast function
    //     self.raw_bytes(raw_array)
    // }
    // }
    pub fn decode(&self, input: Option<&[u8]>) -> OperationsResult<Vec<Option<f64>>> {
        let raw = self.raw_array(input)?;

        Ok(decode_array(raw.data()))
    }

    //     pub fn decode_native(&self, input: Option<&[u8]>) -> OperationsResult<Vec<Option<f64>>> {
    //     let raw = self.raw_array(input)?;

    //     Ok(decode_array(raw.data()))
    // }
    pub fn getpixel(&self, input: Option<&[u8]>, x: u64, y: u64) -> OperationsResult<f64> {
        let decompressed = self.decompress(input).unwrap();
        let data_type = self.metadata().data_type();

        let pixel_size = data_type.size();
        let offset = (y * self.metadata().tile_size() as u64 + x) as usize * pixel_size;

        get_pixel(decompressed, data_type, offset)
    }

    pub fn statistics(&self, input: Option<&[u8]>) -> OperationsResult<TileStatistics> {
        let raw = self.raw_array(input)?;

        let data = match self.no_data() {
            Some(v) => filter_float_array(raw.data(), v),
            None => no_filter_float_array(raw.data()),
        };

        let mut stats: Stats<f64> = Stats::new();
        data.iter().for_each(|v| stats.update(*v));
        let out = TileStatistics {
            min: stats.min,
            max: stats.max,
            mean: stats.mean,
            std_dev: stats.std_dev,
            valid_count: stats.count as u64,
        };
        Ok(out)
    }
}

#[macro_export]
macro_rules! impl_decode_native {
    ($out_array:ident, $out_type:ident, $name:ident) => {
        impl Operations {
            // decode(&self, input: Option<&[u8]>) -> OperationsResult<Vec<Option<f64>>>
            pub fn $name(&self, input: Option<&[u8]>) -> OperationsResult<Vec<Option<$out_type>>> {
                let raw = self.raw_array(input)?;

                Ok($out_array(raw.data()))
            }
        }
    };
}

impl_decode_native!(decode_native_array_i8, i8, decode_native_i8);
impl_decode_native!(decode_native_array_u8, u8, decode_native_u8);

impl_decode_native!(decode_native_array_i16, i16, decode_native_i16);
impl_decode_native!(decode_native_array_u16, u16, decode_native_u16);

impl_decode_native!(decode_native_array_i32, i32, decode_native_i32);
impl_decode_native!(decode_native_array_u32, u32, decode_native_u32);

impl_decode_native!(decode_native_array_i64, i64, decode_native_i64);
impl_decode_native!(decode_native_array_u64, u64, decode_native_u64);

impl_decode_native!(decode_native_array_f32, f32, decode_native_f32);
impl_decode_native!(decode_native_array_f64, f64, decode_native_f64);

// impl Operations {
//     pub fn encode_native_i8(&self, input: Vec<i8>) -> OperationsResult<Vec<u8>> {
//         let typed_array = encode_native_array_i8(input);

//         let encoded_bytes = encode_typed_array(&typed_array)?;

//         let shape = [
//             self.metadata().samples(),
//             self.metadata().tile_size(),
//             self.metadata().tile_size(),
//         ];

//         let raw_array = Array::try_new(encoded_bytes, shape, Some(self.metadata().data_type()))?;

//         // 4. Extract raw uncompressed binary payloads cleanly using your simplified workflow function
//         self.raw_bytes(raw_array)
//     }
// }

#[macro_export]
macro_rules! impl_encode_native {
    ($in_encoder:ident, $typed_array_variant:ident, $in_type:ty, $name:ident) => {
        impl Operations {
            /// Encodes native primitive slices back into raw binary segments,
            /// operating as the exact uncompressed mirror to your decode_native functions.
            pub fn $name(
                &self,
                input: Vec<$in_type>
            ) -> OperationsResult<Vec<u8>> {
                let typed_array = $in_encoder(input);

                let encoded_bytes = encode_typed_array(&typed_array)?;

                let shape = [
                    self.metadata().samples(),
                    self.metadata().tile_size(),
                    self.metadata().tile_size(),
                ];

                let raw_array =
                    Array::try_new(encoded_bytes, shape, Some(self.metadata().data_type()))?;

                // 4. Extract raw uncompressed binary payloads cleanly using your simplified workflow function
                self.raw_bytes(raw_array)
            }
        }
    };
}

// Macro Invocations mapping native primitive slices directly to raw uncompressed byte payloads
impl_encode_native!(encode_native_array_i8, Int8, i8, encode_native_i8);
impl_encode_native!(encode_native_array_u8, UInt8, u8, encode_native_u8);
impl_encode_native!(encode_native_array_i16, Int16, i16, encode_native_i16);
impl_encode_native!(encode_native_array_u16, UInt16, u16, encode_native_u16);
impl_encode_native!(encode_native_array_i32, Int32, i32, encode_native_i32);
impl_encode_native!(encode_native_array_u32, UInt32, u32, encode_native_u32);
impl_encode_native!(encode_native_array_i64, Int64, i64, encode_native_i64);
impl_encode_native!(encode_native_array_u64, UInt64, u64, encode_native_u64);
impl_encode_native!(encode_native_array_f32, Float32, f32, encode_native_f32);
impl_encode_native!(encode_native_array_f64, Float64, f64, encode_native_f64);
