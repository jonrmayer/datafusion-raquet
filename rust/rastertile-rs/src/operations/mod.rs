mod array;
// mod data_type;
mod error;

mod nd_array;

mod statistics;

mod utils;

pub use utils::{decode_array, filter_float_array, get_pixel, no_data, no_filter_float_array};

pub use utils::{
decode_native_array_i8,
decode_native_array_u8,
decode_native_array_i16,
decode_native_array_u16,
decode_native_array_i32,
decode_native_array_u32,
decode_native_array_i64,
decode_native_array_u64,
decode_native_array_f32,
decode_native_array_f64,
};

pub use statistics::TileStatistics;

pub use array::{Array, TypedArray};

// pub use data_type::DataType;
pub use error::{OperationsError, OperationsResult};

use crate::Compression;

use crate::Metadata;

// use ndarray::Array3;

// use nd_array::NdArray;

use rolling_stats::Stats;

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
        let out = no_data(self.metadata().data_type(), self.metadata().no_data());
        out
    }

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
            Some(v) => {
                let f = filter_float_array(raw.data(), v);
                f
            }
            None => {
                let f = no_filter_float_array(raw.data());
                f
            }
        };

        let mut stats: Stats<f64> = Stats::new();
        data.iter().for_each(|v| stats.update(*v));
        let out = TileStatistics {
            min: stats.min as f64,
            max: stats.max as f64,
            mean: stats.mean as f64,
            std_dev: stats.std_dev as f64,
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

impl_decode_native!(decode_native_array_i8,i8,decode_native_i8);
impl_decode_native!(decode_native_array_u8,u8,decode_native_u8);

impl_decode_native!(decode_native_array_i16,i16,decode_native_i16);
impl_decode_native!(decode_native_array_u16,u16,decode_native_u16);

impl_decode_native!(decode_native_array_i32,i32,decode_native_i32);
impl_decode_native!(decode_native_array_u32,u32,decode_native_u32);

impl_decode_native!(decode_native_array_i64,i64,decode_native_i64);
impl_decode_native!(decode_native_array_u64,u64,decode_native_u64);



impl_decode_native!(decode_native_array_f32,f32,decode_native_f32);
impl_decode_native!(decode_native_array_f64,f64,decode_native_f64);
