use crate::DataType;
use crate::operations::TypedArray;
use crate::operations::{OperationsError, OperationsResult};

#[macro_export]
macro_rules! impl_decode_native_array {
    ($in_type:ident, $out_type:ident, $name:ident) => {
        pub fn $name(data: &TypedArray) -> Vec<Option<$out_type>> {
            let vals = match data {
                TypedArray::$in_type(v) => {
                    let out = v.iter().map(|n| Some(*n)).collect();
                    out
                }
                _ => todo!(),
            };
            vals
        }
    };
}

impl_decode_native_array!(Int8, i8, decode_native_array_i8);
impl_decode_native_array!(UInt8, u8, decode_native_array_u8);
impl_decode_native_array!(Int16, i16, decode_native_array_i16);
impl_decode_native_array!(UInt16, u16, decode_native_array_u16);
impl_decode_native_array!(Int32, i32, decode_native_array_i32);
impl_decode_native_array!(UInt32, u32, decode_native_array_u32);
impl_decode_native_array!(Int64, i64, decode_native_array_i64);
impl_decode_native_array!(UInt64, u64, decode_native_array_u64);
impl_decode_native_array!(Float32, f32, decode_native_array_f32);
impl_decode_native_array!(Float64, f64, decode_native_array_f64);