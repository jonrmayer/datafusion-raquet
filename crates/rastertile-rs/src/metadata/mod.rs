use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::fmt;
use std::str::FromStr;
use std::usize;

mod error;
pub use error::{MetadataError, MetadataResult};

use crate::DataType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Hash)]
pub enum BinaryType {
    /// Separated
    #[default]
    Separated,
    /// Interleaved format
    Interleaved,
}

impl fmt::Display for BinaryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinaryType::Separated => write!(f, "Separated"),
            BinaryType::Interleaved => write!(f, "Interleaved"),
        }
    }
}

impl FromStr for BinaryType {
    type Err = MetadataError;
    fn from_str(s: &str) -> MetadataResult<Self> {
        let parser: Option<BinaryType> = match s {
            "Separated" => Some(BinaryType::Separated),
            "Interleaved" => Some(BinaryType::Interleaved),

            _ => todo!(),
        };

        Ok(parser.unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Hash)]
pub enum CompressionFormat {
    /// None
    #[default]
    None,
    /// Gzip format
    Gzip,
    /// Jpeg format
    Jpeg,
    /// WebP format
    WebP,
}

impl fmt::Display for CompressionFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompressionFormat::Gzip => write!(f, "gzip"),
            CompressionFormat::Jpeg => write!(f, "jpeg"),
            CompressionFormat::WebP => write!(f, "webp"),
            CompressionFormat::None => write!(f, "none"),
        }
    }
}

impl FromStr for CompressionFormat {
    type Err = MetadataError;
    fn from_str(s: &str) -> MetadataResult<Self> {
        let parser: Option<CompressionFormat> = match s {
            "gzip" => Some(CompressionFormat::Gzip),
            "jpeg" => Some(CompressionFormat::Jpeg),
            "webp" => Some(CompressionFormat::WebP),
            _ => Some(CompressionFormat::None),
        };

        Ok(parser.unwrap())
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Metadata {
    pub tile_size: usize,
    pub binary_type: BinaryType,
    pub data_type: DataType,
    pub no_data: String,
    pub compression: CompressionFormat,
    pub bands: Option<Vec<String>>,
}

impl Metadata {
    /// Creates a new [`Metadata`] object.
    pub fn new(
        tile_size: usize,
        binary_type: BinaryType,
        data_type: DataType,
        no_data: String,
        compression: CompressionFormat,
        bands: Option<Vec<String>>,
    ) -> Self {
        Self {
            tile_size,
            binary_type,
            data_type,
            no_data,
            compression,
            bands,
        }
    }
    pub fn new_from_strings(
        tile_size_str: String,
        binary_type_str: String,
        data_type_str: String,
        no_data: String,
        compression_str: String,
        bands: Option<Vec<String>>,
    ) -> Self {
        Self {
            tile_size: tile_size_str.parse().expect("Could not convert"),
            binary_type: BinaryType::from_str(&binary_type_str).unwrap(),
            data_type: DataType::from_str(&data_type_str).unwrap(),
            no_data,
            compression: CompressionFormat::from_str(&compression_str).unwrap(),
            bands,
        }
    }

    pub fn bands(&self) -> Option<Vec<String>> {
        self.bands.clone()
    }

    pub fn samples(&self) -> usize {
        match self.bands() {
            Some(bands) => bands.len(),
            _ => 1,
        }
    }

    /// Expose the underlying tile_size.
    pub fn tile_size(&self) -> usize {
        self.tile_size
    }

    /// Expose the underlying binary_type
    pub fn binary_type(&self) -> BinaryType {
        self.binary_type
    }

    /// Expose the underlying binary_type
    pub fn data_type(&self) -> DataType {
        self.data_type
    }

    pub fn no_data(&self) -> String {
        self.no_data.clone()
    }

    /// Expose the underlying binary_type
    pub fn compression(&self) -> CompressionFormat {
        self.compression
    }

    pub fn to_json_value(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }

    pub fn to_str_value(&self) -> String {
        let json_value = self.to_json_value();

        serde_json::to_string(&json_value).unwrap()
    }

    /// Serialize this metadata to a string.
    ///
    /// If `None`, no extension metadata should be written.
    pub fn serialize(&self) -> Option<String> {
        Some(serde_json::to_string(&self).unwrap())
    }

    // /// Deserialize metadata from a string.
    pub fn deserialize<S: AsRef<str>>(metadata: Option<S>) -> Result<Self, MetadataError> {
        if let Some(ext_meta) = metadata {
            Ok(serde_json::from_str(ext_meta.as_ref()).map_err(|_err| {
                MetadataError::General("Bool must be constructed via Array::try_new".to_string())
            })?)
        } else {
            Ok(Default::default())
        }
    }
}
