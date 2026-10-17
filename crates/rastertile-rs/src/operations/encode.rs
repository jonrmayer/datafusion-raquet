use crate::DataType;
use crate::operations::TypedArray;
use crate::operations::{OperationsError, OperationsResult};

/// Encodes an internal `TypedArray` data structure down to a raw uncompressed byte buffer.
/// This functions as the exact inverse conversion layer of the primitive decoder suite.
pub fn encode_typed_array(data: &TypedArray) -> OperationsResult<Vec<u8>> {
    let bytes = match data {
        TypedArray::Int8(v) => bytemuck::cast_slice(v).to_vec(),
        TypedArray::UInt8(v) => v.clone(), // u8 is already raw bytes, clone cleanly
        TypedArray::Int16(v) => bytemuck::cast_slice(v).to_vec(),
        TypedArray::UInt16(v) => bytemuck::cast_slice(v).to_vec(),
        TypedArray::Int32(v) => bytemuck::cast_slice(v).to_vec(),
        TypedArray::UInt32(v) => bytemuck::cast_slice(v).to_vec(),
        TypedArray::Int64(v) => bytemuck::cast_slice(v).to_vec(),
        TypedArray::UInt64(v) => bytemuck::cast_slice(v).to_vec(),
        TypedArray::Float32(v) => bytemuck::cast_slice(v).to_vec(),
        TypedArray::Float64(v) => bytemuck::cast_slice(v).to_vec(),
        _ => return Err(OperationsError::General("Unsupported TypedArray primitive layout variant".into())),
    };
    Ok(bytes)
}

#[macro_export]
macro_rules! impl_encode_native_array {
    ($variant:ident, $inner_type:ty, $name:ident) => {
        pub fn $name(vals: Vec<$inner_type>) -> TypedArray {
            // Reconstruct the raw data vector. 
            // Here, we substitute None with the type's default (e.g., 0 or 0.0)
            // to perfectly match the original vector length.
            // let raw_vec: Vec<$inner_type> = vals
            //     .into_iter()
            //     .map(|opt| opt.unwrap_or_default())
            //     .collect();
                
            TypedArray::$variant(vals)
        }
    };
}

impl_encode_native_array!(Int8, i8, encode_native_array_i8);
impl_encode_native_array!(UInt8, u8, encode_native_array_u8);
impl_encode_native_array!(Int16, i16, encode_native_array_i16);
impl_encode_native_array!(UInt16, u16, encode_native_array_u16);
impl_encode_native_array!(Int32, i32, encode_native_array_i32);
impl_encode_native_array!(UInt32, u32, encode_native_array_u32);
impl_encode_native_array!(Int64, i64, encode_native_array_i64);
impl_encode_native_array!(UInt64, u64, encode_native_array_u64);
impl_encode_native_array!(Float32, f32, encode_native_array_f32);
impl_encode_native_array!(Float64, f64, encode_native_array_f64);
