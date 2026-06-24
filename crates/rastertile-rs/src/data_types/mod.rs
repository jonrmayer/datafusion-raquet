use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::str::FromStr;

mod error;

pub use error::{DataTypeResult,DataTypeError};

/// Supported numeric data types for array elements.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataType {
    /// Boolean mask data.
    Bool,
    /// Unsigned 8-bit integer.
    #[default]
    UInt8,
    /// Unsigned 16-bit integer.
    UInt16,
    /// Unsigned 32-bit integer.
    UInt32,
    /// Unsigned 64-bit integer.
    UInt64,
    /// Signed 8-bit integer.
    Int8,
    /// Signed 16-bit integer.
    Int16,
    /// Signed 32-bit integer.
    Int32,
    /// Signed 64-bit integer.
    Int64,
    /// 32-bit floating point.
    Float32,
    /// 64-bit floating point.
    Float64,
}

impl DataType {
    pub fn size(&self) -> usize {
        match self {
            DataType::Bool | DataType::UInt8 | DataType::Int8 => 1,
            DataType::UInt16 | DataType::Int16 => 2,
            DataType::UInt32 | DataType::Int32 | DataType::Float32 => 4,
            DataType::UInt64 | DataType::Int64 | DataType::Float64 => 8,
        }
    }
}

impl FromStr for DataType {
    type Err = DataTypeError;
    fn from_str(s: &str) -> DataTypeResult<Self> {
        let parser: Option<DataType> = match s {
            "uint8" => Some(DataType::UInt8),
            "int8" => Some(DataType::Int8),
            "uint16" => Some(DataType::UInt16),
            "int16" => Some(DataType::Int16),
            "uint32" => Some(DataType::UInt32),
            "int32" => Some(DataType::Int32),
            "uint64" => Some(DataType::UInt64),
            "int64" => Some(DataType::Int64),
            // "float16" => Some(RasterDataType::Float32),
            "float32" => Some(DataType::Float32),
            "float64" => Some(DataType::Float64),
            _ => None,
        };

        Ok(parser.unwrap())
    }
}
