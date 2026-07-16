use arrow_schema::{ArrowError, Field};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::usize;

use rastertile_rs::Metadata as InnerMetadata;
use rastertile_rs::{BinaryType, CompressionFormat, DataType};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Metadata {
    pub inner: InnerMetadata,
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
        let inner = InnerMetadata::new(
            tile_size,
            binary_type,
            data_type,
            no_data,
            compression,
            bands,
        );
        Self { inner }
    }

    pub fn inner(&self) -> InnerMetadata {
        self.inner.clone()
    }

    pub fn bands(&self) -> Option<Vec<String>> {
        self.inner().bands()
    }

    /// Expose the underlying tile_size.
    pub fn tile_size(&self) -> usize {
        self.inner().tile_size()
    }

    /// Expose the underlying binary_type
    pub fn binary_type(&self) -> BinaryType {
        self.inner().binary_type()
    }

    /// Expose the underlying binary_type
    pub fn data_type(&self) -> DataType {
        self.inner().data_type()
    }

    pub fn no_data(&self) -> String {
        self.inner().no_data()
    }

    /// Expose the underlying binary_type
    pub fn compression(&self) -> CompressionFormat {
        self.inner().compression()
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
    pub(crate) fn serialize(&self) -> Option<String> {
        Some(serde_json::to_string(&self).unwrap())
    }

    /// Deserialize metadata from a string.
    pub(crate) fn deserialize<S: AsRef<str>>(metadata: Option<S>) -> Result<Self, ArrowError> {
        if let Some(ext_meta) = metadata {
            Ok(serde_json::from_str(ext_meta.as_ref())
                .map_err(|err| ArrowError::ExternalError(Box::new(err)))?)
        } else {
            Ok(Default::default())
        }
    }
}

impl TryFrom<&Field> for Metadata {
    type Error = ArrowError;

    fn try_from(value: &Field) -> Result<Self, Self::Error> {
        Self::deserialize(value.extension_type_metadata())
    }
}
