/// Supported numeric data types for array elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    /// Boolean mask data.
    Bool,
    /// Unsigned 8-bit integer.
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
