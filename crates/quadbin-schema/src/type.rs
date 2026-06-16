use std::sync::Arc;

use crate::Metadata;
use arrow_schema::extension::ExtensionType;
use arrow_schema::{ArrowError, DataType, Field};

use crate::error::{QuadbinArrowError, QuadbinArrowResult};

macro_rules! define_basic_type {
    (
        $(#[$($attrss:meta)*])*
        $struct_name:ident
    ) => {
        $(#[$($attrss)*])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $struct_name {
            metadata: Arc<Metadata>,
        }

        impl $struct_name {
            /// Construct a new type from parts.
            pub fn new( metadata: Arc<Metadata>) -> Self {
                Self {
                    metadata,
                }
            }


            /// Change the underlying [`Metadata`]
            pub fn with_metadata(self, metadata: Arc<Metadata>) -> Self {
                Self { metadata, ..self }
            }

            /// Retrieve the underlying [`Metadata`]
            pub fn metadata(&self) -> &Arc<Metadata> {
                &self.metadata
            }

            /// Convert this type to a [`Field`], retaining extension metadata.
            pub fn to_field<N: Into<String>>(&self, name: N, nullable: bool) -> Field {
                Field::new(name, self.data_type(), nullable).with_extension_type(self.clone())
            }

            /// Extract into components
            pub fn into_inner(self) -> Arc<Metadata> {
                ( self.metadata)
            }
        }
    };
}

// define_basic_type!(
//     /// A GeoArrow Point type.
//     ///
//     /// Refer to the [GeoArrow
//     /// specification](https://github.com/geoarrow/geoarrow/blob/main/format.md#point).
//     QuadbinType
// );

// use crate::metadata::Metadata;
/// A GeoArrow WKB type.
///
/// This extension type supports  [`DataType::BinaryView`].
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

// #[cfg(test)]
// mod test {
//     use std::sync::Arc;

//     use arrow_schema::{DataType, Field};

//     use super::*;

//     // use crate::{raquet::RaquetMetadata};
//     // use crate::crs::Crs;
//     // use crate::edges::Edges;

//     const RHEALPIX_JSON: &str = r#"{
//     "file_format": "raquet",
//     "version": "0.5.0",
//     "width": 9216,
//     "height": 7936,
//     "crs": "EPSG:3857",
//     "bounds": [-19.69, 26.43, 5.63, 44.09],
//     "bounds_crs": "EPSG:4326",
//     "compression": "gzip",
//     "tiling": {
//         "scheme": "quadbin",
//         "block_width": 256,
//         "block_height": 256,
//         "min_zoom": 3,
//         "max_zoom": 9,
//         "pixel_zoom": 17,
//         "num_blocks": 1116
//     },
//     "bands": [{
//         "name": "band_1",
//         "description": "Global Horizontal Irradiation",
//         "type": "float32",
//         "nodata": null,
//         "unit": "kWh/m²/day",
//         "colorinterp": "undefined",
//         "STATISTICS_MINIMUM": 0.0,
//         "STATISTICS_MAXIMUM": 6.42,
//         "STATISTICS_MEAN": 0.67,
//         "STATISTICS_STDDEV": 1.63,
//         "STATISTICS_VALID_PERCENT": 100.0,
//         "histogram": {
//             "min": -0.01,
//             "max": 6.17,
//             "buckets": 256,
//             "counts": [55644410, 0, "..."]
//         }
//     }]
// }"#;

//     #[test]
//     fn test_point_interleaved_xy() {

//         // let raquet = RaquetMetadata::from_str(RHEALPIX_JSON);
//         // let metadata = Metadata::new(raquet);

//         // let data_type =
//         //     DataType::FixedSizeList(Arc::new(Field::new("band_1", DataType::BinaryView, false)), 2);
//         // let metadata = Arc::new(Metadata::default());
//         // let type_ = RasterType::try_new(&data_type, metadata).unwrap();

//         // //  let expected = r#"{"crs":"EPSG:4326","crs_type":"authority_code","edges":"spherical"}"#;
//         // assert_eq!(type_.serialize_metadata().as_deref(), Some(RHEALPIX_JSON));

//         // assert_eq!(type_.coord_type, CoordType::Interleaved);
//         // assert_eq!(type_.dim, Dimension::XY);
//         // assert_eq!(type_.serialize_metadata(), None);
//     }

//     // #[test]
//     // fn test_point_separated_xyz() {
//     //     let data_type = DataType::Struct(
//     //         vec![
//     //             Field::new("x", DataType::Float64, false),
//     //             Field::new("y", DataType::Float64, false),
//     //             Field::new("z", DataType::Float64, false),
//     //         ]
//     //         .into(),
//     //     );
//     //     let metadata = Arc::new(Metadata::default());
//     //     let type_ = PointType::try_new(&data_type, metadata).unwrap();

//     //     assert_eq!(type_.coord_type, CoordType::Separated);
//     //     assert_eq!(type_.dim, Dimension::XYZ);
//     //     assert_eq!(type_.serialize_metadata(), None);
//     // }

//     // #[test]
//     // fn test_point_metadata() {
//     //     let data_type =
//     //         DataType::FixedSizeList(Arc::new(Field::new("xy", DataType::Float64, false)), 2);
//     //     let crs = Crs::from_authority_code("EPSG:4326".to_string());
//     //     let metadata = Arc::new(Metadata::new(crs, Some(Edges::Spherical)));
//     //     let type_ = PointType::try_new(&data_type, metadata).unwrap();

//     //     let expected = r#"{"crs":"EPSG:4326","crs_type":"authority_code","edges":"spherical"}"#;
//     //     assert_eq!(type_.serialize_metadata().as_deref(), Some(expected));
//     // }

//     // #[test]
//     // fn geometry_data_type() {
//     //     let typ = GeometryCollectionType::new(Dimension::XY, Default::default());
//     //     dbg!(typ.data_type());
//     // }
// }
