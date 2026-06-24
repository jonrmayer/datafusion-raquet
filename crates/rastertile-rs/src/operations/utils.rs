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

impl_decode_native_array!(Int8,i8,decode_native_array_i8);
impl_decode_native_array!(UInt8,u8,decode_native_array_u8);
impl_decode_native_array!(Int16,i16,decode_native_array_i16);
impl_decode_native_array!(UInt16,u16,decode_native_array_u16);
impl_decode_native_array!(Int32,i32,decode_native_array_i32);
impl_decode_native_array!(UInt32,u32,decode_native_array_u32);
impl_decode_native_array!(Int64,i64,decode_native_array_i64);
impl_decode_native_array!(UInt64,u64,decode_native_array_u64);
impl_decode_native_array!(Float32,f32,decode_native_array_f32);
impl_decode_native_array!(Float64,f64,decode_native_array_f64);


// pub fn decode_native_array_i8(data: &TypedArray) -> Vec<Option<i8>> {
//      let vals = match data {
//         TypedArray::Int8(v) => {
//             let out = v.iter().map(|n| Some(*n )).collect();
//             out
//         }
//         _ => todo!(),
//     };
//     vals

// }

pub fn decode_array(data: &TypedArray) -> Vec<Option<f64>> {
    let vals = match data {
        TypedArray::Int8(v) => {
            let out = v.iter().map(|n| Some(*n as f64)).collect();
            out
        }
        TypedArray::Int16(v) => {
            let out = v.iter().map(|n| Some(*n as f64)).collect();
            out
        }
        TypedArray::Int32(v) => {
            let out = v.iter().map(|n| Some(*n as f64)).collect();
            out
        }
        TypedArray::Int64(v) => {
            let out = v.iter().map(|n| Some(*n as f64)).collect();
            out
        }
        TypedArray::UInt8(v) => {
            let out = v.iter().map(|n| Some(*n as f64)).collect();
            out
        }
        TypedArray::UInt16(v) => {
            let out = v.iter().map(|n| Some(*n as f64)).collect();
            out
        }
        TypedArray::UInt32(v) => {
            let out = v.iter().map(|n| Some(*n as f64)).collect();
            out
        }
        TypedArray::UInt64(v) => {
            let out = v.iter().map(|n| Some(*n as f64)).collect();
            out
        }
        TypedArray::Float32(v) => {
            let out = v.iter().map(|n| Some(*n as f64)).collect();
            out
        }
        TypedArray::Float64(v) => {
            let out = v.iter().map(|n| Some(*n)).collect();
            out
        }
        _ => todo!(),
    };
    vals
}

pub fn filter_float_array(data: &TypedArray, filter: f64) -> Vec<f64> {
    let vals = match data {
        TypedArray::Float32(v) => {
            let out = v
                .iter()
                .map(|n| *n as f64)
                .filter(|&x| x == filter || !x.is_nan())
                .collect();
            out
        }
        TypedArray::Float64(v) => {
            let out = v.iter().map(|n| *n).filter(|&x| x == filter).collect();
            out
        }
        _ => todo!(),
    };
    vals
}

pub fn no_filter_float_array(data: &TypedArray) -> Vec<f64> {
    let vals = match data {
        TypedArray::Float32(v) => {
            let out = v.iter().map(|n| *n as f64).collect();
            out
        }
        TypedArray::Float64(v) => {
            let out = v.iter().map(|n| *n).collect();
            out
        }
        _ => todo!(),
    };
    vals
}

pub fn no_data(data_type: DataType, val: String) -> Option<f64> {
    match data_type {
        DataType::Int8 => {
            let out = match val.parse::<i8>() {
                Ok(val) => Some(val as f64),
                Err(_e) => None,
            };
            out
        }
        DataType::Int16 => {
            let out = match val.parse::<i16>() {
                Ok(val) => Some(val as f64),
                Err(_e) => None,
            };
            out
        }
        DataType::Int32 => {
            let out = match val.parse::<i32>() {
                Ok(val) => Some(val as f64),
                Err(_e) => None,
            };
            out
        }
        DataType::Int64 => {
            let out = match val.parse::<i64>() {
                Ok(val) => Some(val as f64),
                Err(_e) => None,
            };
            out
        }
        DataType::UInt8 => {
            let out = match val.parse::<u8>() {
                Ok(val) => Some(val as f64),
                Err(_e) => None,
            };
            out
        }
        DataType::UInt16 => {
            let out = match val.parse::<u16>() {
                Ok(val) => Some(val as f64),
                Err(_e) => None,
            };
            out
        }
        DataType::UInt32 => {
            let out = match val.parse::<u32>() {
                Ok(val) => Some(val as f64),
                Err(_e) => None,
            };
            out
        }
        DataType::UInt64 => {
            let out = match val.parse::<u64>() {
                Ok(val) => Some(val as f64),
                Err(_e) => None,
            };
            out
        }
        DataType::Float32 => {
            let out = match val.parse::<f32>() {
                Ok(val) => Some(val as f64),
                Err(_e) => None,
            };
            out
        }
        DataType::Float64 => {
            let out = match val.parse::<f64>() {
                Ok(val) => Some(val),
                Err(_e) => None,
            };
            out
        }
        _ => todo!(),
    }
}

pub fn get_pixel(data: Vec<u8>, data_type: DataType, offset: usize) -> OperationsResult<f64> {
    let value = match data_type {
        DataType::UInt8 => f64::from(data[offset]),
        DataType::Int8 => f64::from(data[offset] as i8),
        DataType::UInt16 => {
            let bytes: [u8; 2] = data[offset..offset + 2]
                .try_into()
                .map_err(|_| OperationsError::General("Invalid slice length".to_string()))?;
            f64::from(u16::from_ne_bytes(bytes))
        }
        DataType::Int16 => {
            let bytes: [u8; 2] = data[offset..offset + 2]
                .try_into()
                .map_err(|_| OperationsError::General("Invalid slice length".to_string()))?;
            f64::from(i16::from_ne_bytes(bytes))
        }
        DataType::UInt32 => {
            let bytes: [u8; 4] = data[offset..offset + 4]
                .try_into()
                .map_err(|_| OperationsError::General("Invalid slice length".to_string()))?;
            f64::from(u32::from_ne_bytes(bytes))
        }
        DataType::Int32 => {
            let bytes: [u8; 4] = data[offset..offset + 4]
                .try_into()
                .map_err(|_| OperationsError::General("Invalid slice length".to_string()))?;
            f64::from(i32::from_ne_bytes(bytes))
        }
        DataType::Float32 => {
            let bytes: [u8; 4] = data[offset..offset + 4]
                .try_into()
                .map_err(|_| OperationsError::General("Invalid slice length".to_string()))?;
            f64::from(f32::from_ne_bytes(bytes))
        }
        DataType::Float64 => {
            let bytes: [u8; 8] = data[offset..offset + 8]
                .try_into()
                .map_err(|_| OperationsError::General("Invalid slice length".to_string()))?;
            f64::from_ne_bytes(bytes)
        }
        DataType::UInt64 => {
            let bytes: [u8; 8] = data[offset..offset + 8]
                .try_into()
                .map_err(|_| OperationsError::General("Invalid slice length".to_string()))?;
            u64::from_ne_bytes(bytes) as f64
        }
        DataType::Int64 => {
            let bytes: [u8; 8] = data[offset..offset + 8]
                .try_into()
                .map_err(|_| OperationsError::General("Invalid slice length".to_string()))?;
            i64::from_ne_bytes(bytes) as f64
        }
        _ => todo!(),
    };

    Ok(value)
}

// #[macro_export]
// macro_rules! impl_decode_array {
//     ($in_type:ident, $name:ident) => {
//         pub fn $name(data: &TypedArray) -> Vec<Option<$out_type>> {
//             let vals = match data {
//                 TypedArray::$in_type(v) => {
//                     let out = v.iter().map(|n| Some(n.clone())).collect();
//                     out
//                 }
//                 _ => panic!("expected Float32"),
//             };
//             vals
//         }
//     };
// }

// impl_convert_array!(Float32,  convert_f32);
