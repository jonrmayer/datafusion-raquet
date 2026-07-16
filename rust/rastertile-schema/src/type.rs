use std::sync::Arc;

use crate::Metadata;

use arrow_schema::extension::ExtensionType;
use arrow_schema::{ArrowError, DataType};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct RasterType {
    metadata: Arc<Metadata>,
}

impl RasterType {
    /// Construct a new type from parts.
    pub fn new(metadata: Arc<Metadata>) -> Self {
        Self { metadata }
    }

    /// Change the underlying [`Metadata`]
    pub fn with_metadata(self, metadata: Arc<Metadata>) -> Self {
        Self { metadata }
    }

    /// Retrieve the underlying [`Metadata`]
    pub fn metadata(&self) -> &Arc<Metadata> {
        &self.metadata
    }
}

impl ExtensionType for RasterType {
    const NAME: &'static str = "rasterarrow.raster";

    type Metadata = Arc<Metadata>;

    fn metadata(&self) -> &Self::Metadata {
        self.metadata()
    }

    fn serialize_metadata(&self) -> Option<String> {
        self.metadata.serialize()
    }

    fn deserialize_metadata(metadata: Option<&str>) -> Result<Self::Metadata, ArrowError> {
        Ok(Arc::new(Metadata::deserialize(metadata)?))
    }

    fn supports_data_type(&self, data_type: &DataType) -> Result<(), ArrowError> {
        match data_type {
            DataType::Binary | DataType::LargeBinary | DataType::BinaryView => Ok(()),
            dt => Err(ArrowError::SchemaError(format!(
                "Unexpected data type {dt}"
            ))),
        }
    }

    fn try_new(data_type: &DataType, metadata: Self::Metadata) -> Result<Self, ArrowError> {
        let raster = Self { metadata };
        raster.supports_data_type(data_type)?;
        Ok(raster)
    }
}
