//! Contains the implementation of [`GeoArrowType`], which defines all geometry arrays in this
//! crate.

use std::sync::Arc;

use arrow_schema::extension::ExtensionType;
use arrow_schema::{DataType, Field};

use crate::Metadata;

use crate::error::{RasterArrowError, RasterArrowResult};
use crate::{ RasterType,RasterFloat32Type};

/// Geospatial data types supported by GeoArrow.
///
/// The variants of this enum include all possible GeoArrow geometry types, including both "native"
/// and "serialized" encodings.
///
/// Each variant uniquely identifies the physical buffer layout for the respective array type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RasterArrowType {
    /// A Raster stored in a `BinaryArray` with `i32` offsets.
    Raster(RasterType),

    /// A Raster stored in a `LargeBinaryArray` with `i64` offsets.
    LargeRaster(RasterType),

    /// A Raster stored in a `BinaryViewArray`.
    RasterView(RasterType),

    RasterF32(RasterFloat32Type),
}

impl From<RasterArrowType> for DataType {
    fn from(value: RasterArrowType) -> Self {
        value.to_data_type()
    }
}

impl RasterArrowType {
    /// Returns the [Metadata] contained within this type.
    pub fn metadata(&self) -> &Arc<Metadata> {
        use RasterArrowType::*;
        match self {
            RasterF32(t) => t.metadata(),
            Raster(t) | LargeRaster(t) | RasterView(t)  => t.metadata(),
           
        }
    }
    /// Converts a [`GeoArrowType`] into the relevant arrow [`DataType`].
    ///
    /// Note that an arrow [`DataType`] will lose the accompanying GeoArrow metadata if it is not
    /// part of a [`Field`] with GeoArrow extension metadata in its field metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// # use arrow_schema::DataType;
    /// # use geoarrow_schema::{Dimension, GeoArrowType, PointType};
    /// #
    /// let point_type = PointType::new(Dimension::XY, Default::default());
    /// let data_type = GeoArrowType::Point(point_type).to_data_type();
    /// assert!(matches!(data_type, DataType::Struct(_)));
    /// ```
    pub fn to_data_type(&self) -> DataType {
        use RasterArrowType::*;
        match self {
            Raster(_) => DataType::Binary,
            LargeRaster(_) => DataType::LargeBinary,
            RasterView(_) => DataType::BinaryView,
            RasterF32(t) => t.data_type(),
        }
    }


    pub fn to_field<N: Into<String>>(&self, name: N, nullable: bool) -> Field {
        use RasterArrowType::*;
        match self {
           
            Raster(t) | LargeRaster(t) | RasterView(t) => {
                Field::new(name, self.to_data_type(), nullable).with_extension_type(t.clone())
            },
            RasterF32(t) => t.to_field(name, nullable)
           
        }
    }



    /// Applies the provided [Metadata] onto self.
    pub fn with_metadata(self, meta: Arc<Metadata>) -> RasterArrowType {
        use RasterArrowType::*;
        match self {
           
            Raster(t) => Raster(t.with_metadata(meta)),
            LargeRaster(t) => LargeRaster(t.with_metadata(meta)),
            RasterView(t) => RasterView(t.with_metadata(meta)),
            RasterF32(t) => RasterF32(t.with_metadata(meta)),
          
        }
    }

    /// Create a new [`GeoArrowType`] from an Arrow [`Field`], requiring GeoArrow metadata to be
    /// set.
    ///
    /// If the field does not have at least a GeoArrow extension name, an error will be returned.
    ///
    /// See also [`GeoArrowType::from_arrow_field`].
    pub fn from_extension_field(field: &Field) -> RasterArrowResult<Self> {
        let extension_name = field.extension_type_name().ok_or(RasterArrowError::InvalidGeoArrow(
                "Expected GeoArrow extension metadata, but found none, and `require_geoarrow_metadata` is `true`.".to_string(),
            ))?;

        use RasterArrowType::*;
        let data_type = match extension_name {
            RasterFloat32Type::NAME => RasterF32(field.try_extension_type()?),

            RasterType::NAME => match field.data_type() {
                
                DataType::Binary => Raster(field.try_extension_type()?),
                DataType::LargeBinary => LargeRaster(field.try_extension_type()?),
                DataType::BinaryView => RasterView(field.try_extension_type()?),
                _ => {
                    return Err(RasterArrowError::InvalidGeoArrow(format!(
                        "Expected binary type for a field with extension name 'geoarrow.Raster', got '{}'",
                        field.data_type()
                    )));
                }
            },
          
            name => {
                return Err(RasterArrowError::InvalidGeoArrow(format!(
                    "Expected a GeoArrow extension name, got an Arrow extension type with name: '{name}'.",
                )));
            }
        };
        Ok(data_type)
    }

  
    pub fn from_arrow_field(field: &Field) -> RasterArrowResult<Self> {
        use RasterArrowType::*;
        if let Ok(geo_type) = Self::from_extension_field(field) {
            Ok(geo_type)
        } else {
            let metadata = Arc::new(Metadata::try_from(field)?);
            let data_type = match field.data_type() {
 
                DataType::Binary => Raster(RasterType::new(metadata)),
                DataType::LargeBinary => LargeRaster(RasterType::new(metadata)),
                DataType::BinaryView => RasterView(RasterType::new(metadata)),
               
                _ => return Err(RasterArrowError::InvalidGeoArrow("Only FixedSizeList, Struct, Binary, LargeBinary, BinaryView, String, LargeString, and StringView arrays are unambigously typed for a GeoArrow type and can be used without extension metadata.\nEnsure your array input has GeoArrow metadata.".to_string())),
            };

            Ok(data_type)
        }
    }
}


macro_rules! impl_into_geoarrowtype {
    ($source_type:ident, $variant:expr) => {
        impl From<$source_type> for RasterArrowType {
            fn from(value: $source_type) -> Self {
                $variant(value)
            }
        }
    };
}

impl_into_geoarrowtype!(RasterFloat32Type, RasterArrowType::RasterF32);

impl TryFrom<&Field> for RasterArrowType {
    type Error = RasterArrowError;

    fn try_from(field: &Field) -> RasterArrowResult<Self> {
        Self::from_extension_field(field)
    }
}
