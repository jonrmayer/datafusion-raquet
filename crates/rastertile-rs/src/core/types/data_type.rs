use core::fmt;

use serde::{Deserialize, Serialize};

/// Raster data types representing pixel values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
#[derive(Default)]
pub enum RasterDataType {
    /// Unsigned 8-bit integer (0-255)
    #[default]
    UInt8 = 1,
    /// Signed 8-bit integer (-128 to 127)
    Int8 = 2,
    /// Unsigned 16-bit integer (0-65535)
    UInt16 = 3,
    /// Signed 16-bit integer
    Int16 = 4,
    /// Unsigned 32-bit integer
    UInt32 = 5,
    /// Signed 32-bit integer
    Int32 = 6,
    /// Unsigned 64-bit integer
    UInt64 = 7,
    /// Signed 64-bit integer
    Int64 = 8,
    // /// 16-bit floating point - for ML/inference use cases
    // Float16 = 9,
    /// 32-bit floating point
    Float32 = 9,
    /// 64-bit floating point
    Float64 = 10,
}

impl RasterDataType {
    /// Returns the size in bytes of this data type
    #[must_use]
    pub const fn size_bytes(self) -> usize {
        match self {
            Self::UInt8 | Self::Int8 => 1,
            Self::UInt16 | Self::Int16 => 2,
            Self::UInt32 | Self::Int32 | Self::Float32 => 4,
            Self::UInt64 | Self::Int64 | Self::Float64 => 8,
        }
    }

    /// Returns the size in bits of this data type
    #[must_use]
    pub const fn size_bits(self) -> usize {
        self.size_bytes() * 8
    }

    /// Returns true if this is a signed type
    #[must_use]
    pub const fn is_signed(self) -> bool {
        matches!(
            self,
            Self::Int8
                | Self::Int16
                | Self::Int32
                | Self::Int64
                // | Self::Float16
                | Self::Float32
                | Self::Float64
        )
    }

    /// Returns true if this is a floating-point type
    #[must_use]
    pub const fn is_floating_point(self) -> bool {
        matches!(self, Self::Float32 | Self::Float64)
    }
    /// Returns true if this is an integer type
    #[must_use]
    pub const fn is_integer(self) -> bool {
        !self.is_floating_point()
    }

    /// Returns the minimum value for this data type as f64
    #[must_use]
    pub const fn min_value(self) -> f64 {
        match self {
            Self::UInt8 => 0.0,
            Self::Int8 => i8::MIN as f64,
            Self::UInt16 => 0.0,
            Self::Int16 => i16::MIN as f64,
            Self::UInt32 => 0.0,
            Self::Int32 => i32::MIN as f64,
            Self::UInt64 => 0.0,
            Self::Int64 => i64::MIN as f64,
            Self::Float32 => f32::MIN as f64,
            Self::Float64 => f64::MIN,
        }
    }

    /// Returns the maximum value for this data type as f64
    #[must_use]
    pub const fn max_value(self) -> f64 {
        match self {
            Self::UInt8 => u8::MAX as f64,
            Self::Int8 => i8::MAX as f64,
            Self::UInt16 => u16::MAX as f64,
            Self::Int16 => i16::MAX as f64,
            Self::UInt32 => u32::MAX as f64,
            Self::Int32 => i32::MAX as f64,
            Self::UInt64 => u64::MAX as f64,
            Self::Int64 => i64::MAX as f64,
            Self::Float32 => f32::MAX as f64,
            Self::Float64 => f64::MAX,
        }
    }

     /// Returns the name of this data type
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::UInt8 => "UInt8",
            Self::Int8 => "Int8",
            Self::UInt16 => "UInt16",
            Self::Int16 => "Int16",
            Self::UInt32 => "UInt32",
            Self::Int32 => "Int32",
            Self::UInt64 => "UInt64",
            Self::Int64 => "Int64",
            Self::Float32 => "Float32",
            Self::Float64 => "Float64",
           
        }
    }
}

impl fmt::Display for RasterDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// `NoData` value representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum NoDataValue {
    /// No `NoData` value defined
    #[default]
    None,
    /// Integer `NoData` value
    Integer(i64),
    /// Floating-point `NoData` value
    Float(f64),
}

impl NoDataValue {
    /// Returns the `NoData` value as f64, if defined
    #[must_use]
    pub const fn as_f64(&self) -> Option<f64> {
        match self {
            Self::None => None,
            Self::Integer(v) => Some(*v as f64),
            Self::Float(v) => Some(*v),
        }
    }

    /// Returns true if this represents "no `NoData`"
    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Creates a `NoData` value from an integer
    #[must_use]
    pub const fn from_integer(value: i64) -> Self {
        Self::Integer(value)
    }

    /// Creates a `NoData` value from a float
    #[must_use]
    pub const fn from_float(value: f64) -> Self {
        Self::Float(value)
    }
}
