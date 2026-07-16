use arrow_schema::{ArrowError, Field};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::usize;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Metadata {
    pub min_zoom: i32,
    pub max_zoom: i32,
}

impl Metadata {
    /// Creates a new [`Metadata`] object.
    pub fn new(min_zoom: i32, max_zoom: i32) -> Self {
        Self { min_zoom, max_zoom }
    }

    /// Expose the underlying tile_size.
    pub fn min_zoom(&self) -> &i32 {
        &self.min_zoom
    }

    /// Expose the underlying binary_type
    pub fn max_zoom(&self) -> &i32 {
        &self.max_zoom
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
