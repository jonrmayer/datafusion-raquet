use std::sync::Arc;

use crate::Metadata;
use arrow_schema::extension::ExtensionType;
use arrow_schema::{ArrowError, DataType, Field};

use crate::error::{QuadbinArrowError, QuadbinArrowResult};

// macro_rules! define_basic_type {
//     (
//         $(#[$($attrss:meta)*])*
//         $struct_name:ident
//     ) => {
//         $(#[$($attrss)*])*
//         #[derive(Debug, Clone, PartialEq, Eq, Hash)]
//         pub struct $struct_name {
//             metadata: Arc<Metadata>,
//         }

//         impl $struct_name {
//             /// Construct a new type from parts.
//             pub fn new( metadata: Arc<Metadata>) -> Self {
//                 Self {
//                     metadata,
//                 }
//             }

//             /// Change the underlying [`Metadata`]
//             pub fn with_metadata(self, metadata: Arc<Metadata>) -> Self {
//                 Self { metadata, ..self }
//             }

//             /// Retrieve the underlying [`Metadata`]
//             pub fn metadata(&self) -> &Arc<Metadata> {
//                 &self.metadata
//             }

//             /// Convert this type to a [`Field`], retaining extension metadata.
//             pub fn to_field<N: Into<String>>(&self, name: N, nullable: bool) -> Field {
//                 Field::new(name, self.data_type(), nullable).with_extension_type(self.clone())
//             }

//             /// Extract into components
//             pub fn into_inner(self) -> Arc<Metadata> {
//                 ( self.metadata)
//             }
//         }
//     };
// }

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct QuadbinType {
    metadata: Arc<Metadata>,
}

impl QuadbinType {
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

    pub fn to_data_type(&self) -> DataType {
        DataType::UInt64
    }

    pub fn to_field<N: Into<String>>(&self, name: N, nullable: bool) -> Field {
        Field::new(name, self.to_data_type(), nullable).with_extension_type(self.clone())
    }

    pub fn from_extension_field(field: &Field) -> QuadbinArrowResult<Self> {
        let extension_name = field.extension_type_name().ok_or(QuadbinArrowError::InvalidGeoArrow(
                "Expected rasterarrow extension metadata, but found none, and `require_rasterarrow_metadata` is `true`.".to_string(),
            ))?;

        let data_type = match extension_name {
            QuadbinType::NAME => match field.data_type() {
                DataType::UInt64 => field.try_extension_type()?,
                _ => {
                    return Err(QuadbinArrowError::InvalidGeoArrow(format!(
                        "Expected binary type for a field with extension name 'geoarrow.wkb', got '{}'",
                        field.data_type()
                    )));
                }
            },
            name => {
                return Err(QuadbinArrowError::InvalidGeoArrow(format!(
                    "Expected a GeoArrow extension name, got an Arrow extension type with name: '{name}'.",
                )));
            }
        };
        Ok(data_type)
    }

    pub fn from_arrow_field(field: &Field) -> QuadbinArrowResult<Self> {
        if let Ok(geo_type) = Self::from_extension_field(field) {
            Ok(geo_type)
        } else {
            let metadata = Arc::new(Metadata::try_from(field)?);
            let data_type = match field.data_type() {
                DataType::UInt64 => QuadbinType::new(metadata),
                _ => return Err(QuadbinArrowError::InvalidGeoArrow("Only FixedSizeList, Struct, Binary, LargeBinary, BinaryView, String, LargeString, and StringView arrays are unambigously typed for a GeoArrow type and can be used without extension metadata.\nEnsure your array input has GeoArrow metadata.".to_string())),
             };
            Ok(data_type)
        }
    }
}

impl ExtensionType for QuadbinType {
    const NAME: &'static str = "rasterarrow.quadbin";

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
            DataType::UInt64 => Ok(()),
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

impl From<QuadbinType> for DataType {
    fn from(value: QuadbinType) -> Self {
        value.to_data_type()
    }
}

impl TryFrom<&Field> for QuadbinType {
    type Error = QuadbinArrowError;

    fn try_from(field: &Field) -> QuadbinArrowResult<Self> {
        Self::from_extension_field(field)
    }
}
