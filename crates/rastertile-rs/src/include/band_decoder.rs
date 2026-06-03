use std::{fmt, usize};
use std::str::FromStr;

use crate::{Error, Result};

#[derive(Clone, Debug, PartialEq)]
pub enum RasterDataType  {
    UINT8,
    INT8,
    UINT16,
    INT16,
    UINT32,
    INT32,
    UINT64,
    INT64,
    FLOAT16, // new in v0.3.0 for ML/inference use cases
    FLOAT32,
    FLOAT64,
}

impl FromStr for RasterDataType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let parser: Option<RasterDataType> = match s {
            "uint8" => Some(RasterDataType::UINT8),
            "int8" => Some(RasterDataType::INT8),
            "uint16" => Some(RasterDataType::UINT16),
            "int16" => Some(RasterDataType::INT16),
            "uint32" => Some(RasterDataType::UINT32),
            "int32" => Some(RasterDataType::INT32),
            "uint64" => Some(RasterDataType::UINT64),
            "int64" => Some(RasterDataType::INT64),
            "float16" => Some(RasterDataType::FLOAT16),
            "float32" => Some(RasterDataType::FLOAT32),
            "float64" => Some(RasterDataType::FLOAT64),
            _ => None,
        };

        Ok(parser.unwrap())
    }
}

impl fmt::Display for RasterDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RasterDataType::UINT8 => "uint8".to_string(),
            RasterDataType::INT8 => "int8".to_string(),
            RasterDataType::UINT16 => "uint16".to_string(),
            RasterDataType::INT16 => "int16".to_string(),
            RasterDataType::UINT32 => "uint32".to_string(),
            RasterDataType::INT32 => "int32".to_string(),
            RasterDataType::UINT64 => "uint64".to_string(),
            RasterDataType::INT64 => "int64".to_string(),
            RasterDataType::FLOAT16 => "float16".to_string(),
            RasterDataType::FLOAT32 => "float32".to_string(),
            RasterDataType::FLOAT64 => "float64".to_string(),
        };
        f.write_str(&s)
    }
}

pub fn parse_dtype(dtype: &str) -> RasterDataType {
    RasterDataType::from_str(dtype).unwrap()
}

pub fn dtype_size(dtype: RasterDataType) -> usize {
    let size: usize = match dtype {
        RasterDataType::UINT8 | RasterDataType::INT8 => 1,
        RasterDataType::UINT16 | RasterDataType::INT16 | RasterDataType::FLOAT16 => 2,
        RasterDataType::UINT32 | RasterDataType::INT32 | RasterDataType::FLOAT32 => 4,
        RasterDataType::UINT64 | RasterDataType::INT64 | RasterDataType::FLOAT64 => 8,
    };
    size
}

pub fn get_pixel_value(data: Vec<u8>, data_size: usize, offset: usize, dtype: RasterDataType) -> f64 {
    let elem_size = dtype_size(dtype.clone());
    let byte_offset = offset * elem_size;
    if (byte_offset + elem_size) > data_size {}
    let val = match dtype {
        RasterDataType::UINT8 => data[offset] as f64,
        RasterDataType::INT8 => data[offset] as f64,
        RasterDataType::UINT16 => data[offset] as f64,
        RasterDataType::INT16 => data[offset] as f64,
        RasterDataType::UINT32 => data[offset] as f64,
        RasterDataType::INT32 => data[offset] as f64,
        RasterDataType::UINT64 => data[offset] as f64,
        RasterDataType::INT64 => data[offset] as f64,
        RasterDataType::FLOAT16 => data[offset] as f64,
        RasterDataType::FLOAT32 => data[offset] as f64,
        RasterDataType::FLOAT64 => data[offset] as f64,
    };
    val
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dtype() {
        let dtype = "float64";
        let RasterDataType = parse_dtype(dtype);

        let aaa = RasterDataType.to_string();

        println!("RasterDataType {:?} aaa {:?}", RasterDataType, aaa);
    }
    #[test]
    fn test_dtype_size() {
        let dtype = "float64";
        let band_data_type = parse_dtype(dtype);
        let size = dtype_size(band_data_type.clone());
        println!("band_data_type {:?} size {:?}", band_data_type, size);
    }
}
