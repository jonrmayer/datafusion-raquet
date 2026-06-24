use serde::{Deserialize, Serialize};
use serde_json::Value;

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

    pub fn bands(&self) -> Option<Vec<String>> {
        self.bands.clone()
    }

    pub fn samples(&self) -> usize {
        let samples = match self.bands() {
            Some(bands) => bands.len(),
            _ => 1,
        };
        samples
    }

    /// Expose the underlying tile_size.
    pub fn tile_size(&self) -> usize {
        self.tile_size.clone()
    }

    /// Expose the underlying binary_type
    pub fn binary_type(&self) -> BinaryType {
        self.binary_type.clone()
    }

    /// Expose the underlying binary_type
    pub fn data_type(&self) -> DataType {
        self.data_type.clone()
    }

    pub fn no_data(&self) -> String {
        self.no_data.clone()
    }

    /// Expose the underlying binary_type
    pub fn compression(&self) -> CompressionFormat {
        self.compression.clone()
    }

    pub fn to_json_value(&self) -> Value {
        let val = serde_json::to_value(self).unwrap();
        val
    }

    pub fn to_str_value(&self) -> String {
        let json_value = self.to_json_value();
        let val = serde_json::to_string(&json_value).unwrap();
        val
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
            Ok(serde_json::from_str(ext_meta.as_ref()).map_err(|err| {
                MetadataError::General("Bool must be constructed via Array::try_new".to_string())
            })?)
        } else {
            Ok(Default::default())
        }
    }
}
