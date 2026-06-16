use arrow_schema::{ArrowError, Field};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::str::FromStr;
use std::usize;



// use rastertile_rs::{BinaryType,RasterDataType,CompressionFormat};


/// GeoArrow extension metadata.
///
/// This follows the extension metadata [defined by the GeoArrow
/// specification](https://rhealpixarrow.org/extension-types).
///
/// This struct is contained within all GeoArrow geometry type definitions, such as
/// [`PointType`][crate::PointType], [`GeometryType`][crate::GeometryType], or
/// [`WkbType`][crate::WkbType].
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Metadata {
    pub min_zoom: i32,
    pub max_zoom: i32,   
}

impl Metadata {
    /// Creates a new [`Metadata`] object.
    pub fn new(
        min_zoom: i32,
        max_zoom: i32,
      
    ) -> Self {
        Self {
            min_zoom,
            max_zoom,         
        }
    }

    /// Expose the underlying tile_size.
    pub fn min_zoom(&self) -> &i32 {
        &self.min_zoom
    }

    /// Expose the underlying binary_type
    pub fn max_zoom(&self) -> &i32 {
        &self.max_zoom
    }

    // /// Expose the underlying binary_type
    // pub fn data_type(&self) -> &RasterDataType {
    //     &self.data_type
    // }

    // /// Expose the underlying binary_type
    // pub fn compression(&self) -> &CompressionFormat {
    //     &self.compression
    // }

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

// #[cfg(test)]
// mod test {
//     use std::collections::HashMap;
//     use std::str::FromStr;

//     use arrow_schema::DataType;
//     use serde_json::{Value, json};

//     use super::*;

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
//     fn test_crs_authority_code() {
//         // let raquet = RaquetMetadata::from_str(RHEALPIX_JSON);
//         let metadata = Metadata::new(
//             256,
//             BinaryType::Separated,
//             RasterDataType::UInt8,
//             CompressionFormat::None,
//         );

//         // let expected = r#"{"crs":"EPSG:4326","crs_type":"authority_code","edges":"spherical"}"#;
//         let serialized = metadata.serialize();
//         println!("{:?}", serialized);
//         // assert_eq!(serialized.as_deref(), Some(expected));

//         // assert_eq!(
//         //     metadata,
//         //     Metadata::deserialize(serialized.as_deref()).unwrap()
//         // );
//     }

//     // #[test]
//     // fn test_crs_authority_code_no_edges() {
//     //     let crs = Crs::from_authority_code("EPSG:4326".to_string());
//     //     let metadata = Metadata::new(crs, None);

//     //     let expected = r#"{"crs":"EPSG:4326","crs_type":"authority_code"}"#;

//     //     let serialized = metadata.serialize();
//     //     assert_eq!(serialized.as_deref(), Some(expected));

//     //     assert_eq!(
//     //         metadata,
//     //         Metadata::deserialize(serialized.as_deref()).unwrap()
//     //     );
//     // }

//     // #[test]
//     // fn test_crs_wkt() {
//     //     let crs = Crs::from_wkt2_2019(EPSG_4326_WKT.to_string());
//     //     let metadata = Metadata::new(crs, None);

//     //     let expected = r#"{"crs":"GEOGCRS[\"WGS 84\",ENSEMBLE[\"World Geodetic System 1984 ensemble\",MEMBER[\"World Geodetic System 1984 (Transit)\"],MEMBER[\"World Geodetic System 1984 (G730)\"],MEMBER[\"World Geodetic System 1984 (G873)\"],MEMBER[\"World Geodetic System 1984 (G1150)\"],MEMBER[\"World Geodetic System 1984 (G1674)\"],MEMBER[\"World Geodetic System 1984 (G1762)\"],MEMBER[\"World Geodetic System 1984 (G2139)\"],ELLIPSOID[\"WGS 84\",6378137,298.257223563,LENGTHUNIT[\"metre\",1]],ENSEMBLEACCURACY[2.0]],PRIMEM[\"Greenwich\",0,ANGLEUNIT[\"degree\",0.0174532925199433]],CS[ellipsoidal,2],AXIS[\"geodetic latitude (Lat)\",north,ORDER[1],ANGLEUNIT[\"degree\",0.0174532925199433]],AXIS[\"geodetic longitude (Lon)\",east,ORDER[2],ANGLEUNIT[\"degree\",0.0174532925199433]],USAGE[SCOPE[\"Horizontal component of 3D system.\"],AREA[\"World.\"],BBOX[-90,-180,90,180]],ID[\"EPSG\",4326]]","crs_type":"wkt2:2019"}"#;

//     //     let serialized = metadata.serialize();
//     //     assert_eq!(serialized.as_deref(), Some(expected));

//     //     assert_eq!(
//     //         metadata,
//     //         Metadata::deserialize(serialized.as_deref()).unwrap()
//     //     );
//     // }

//     // #[test]
//     // fn test_projjson() {
//     //     let crs = Crs::from_projjson(Value::from_str(EPSG_4326_PROJJSON).unwrap());
//     //     let metadata = Metadata::new(crs, None);

//     //     let expected = r#"{"crs":{"$schema":"https://proj.org/schemas/v0.7/projjson.schema.json","type":"GeographicCRS","name":"WGS 84","datum_ensemble":{"name":"World Geodetic System 1984 ensemble","members":[{"name":"World Geodetic System 1984 (Transit)","id":{"authority":"EPSG","code":1166}},{"name":"World Geodetic System 1984 (G730)","id":{"authority":"EPSG","code":1152}},{"name":"World Geodetic System 1984 (G873)","id":{"authority":"EPSG","code":1153}},{"name":"World Geodetic System 1984 (G1150)","id":{"authority":"EPSG","code":1154}},{"name":"World Geodetic System 1984 (G1674)","id":{"authority":"EPSG","code":1155}},{"name":"World Geodetic System 1984 (G1762)","id":{"authority":"EPSG","code":1156}},{"name":"World Geodetic System 1984 (G2139)","id":{"authority":"EPSG","code":1309}}],"ellipsoid":{"name":"WGS 84","semi_major_axis":6378137,"inverse_flattening":298.257223563},"accuracy":"2.0","id":{"authority":"EPSG","code":6326}},"coordinate_system":{"subtype":"ellipsoidal","axis":[{"name":"Geodetic latitude","abbreviation":"Lat","direction":"north","unit":"degree"},{"name":"Geodetic longitude","abbreviation":"Lon","direction":"east","unit":"degree"}]},"scope":"Horizontal component of 3D system.","area":"World.","bbox":{"south_latitude":-90,"west_longitude":-180,"north_latitude":90,"east_longitude":180},"id":{"authority":"EPSG","code":4326}},"crs_type":"projjson"}"#;

//     //     let serialized = metadata.serialize();

//     //     // We use Value for equality checking because JSON string formatting is different
//     //     assert_eq!(
//     //         Value::from_str(serialized.as_deref().unwrap()).unwrap(),
//     //         Value::from_str(expected).unwrap()
//     //     );

//     //     assert_eq!(
//     //         metadata,
//     //         Metadata::deserialize(serialized.as_deref()).unwrap()
//     //     );
//     // }

//     // #[test]
//     // fn test_unknown_crs() {
//     //     let crs = Crs::from_unknown_crs_type("CRS".to_string());
//     //     let metadata = Metadata::new(crs, None);

//     //     let expected = r#"{"crs":"CRS"}"#;

//     //     let serialized = metadata.serialize();
//     //     assert_eq!(serialized.as_deref(), Some(expected));

//     //     assert_eq!(
//     //         metadata,
//     //         Metadata::deserialize(serialized.as_deref()).unwrap()
//     //     );
//     // }

//     // #[test]
//     // fn test_empty_metadata() {
//     //     let metadata = Metadata::default();
//     //     let serialized = metadata.serialize();
//     //     assert_eq!(serialized.as_deref(), None);

//     //     assert_eq!(
//     //         metadata,
//     //         Metadata::deserialize(serialized.as_deref()).unwrap()
//     //     );
//     // }

//     // #[test]
//     // fn from_field() {
//     //     let field = Field::new("", DataType::Null, false).with_metadata(HashMap::from([(
//     //         "ARROW:extension:metadata".to_string(),
//     //         r#"{"crs": {}, "crs_type": "projjson", "edges": "spherical"}"#.to_string(),
//     //     )]));

//     //     let metadata = Metadata::try_from(&field).unwrap();
//     //     assert_eq!(metadata.crs(), &Crs::from_projjson(json!({})));
//     //     assert_eq!(metadata.edges(), Some(Edges::Spherical));

//     //     let bad_field = Field::new("", DataType::Null, false).with_metadata(HashMap::from([(
//     //         "ARROW:extension:metadata".to_string(),
//     //         "not valid json".to_string(),
//     //     )]));
//     //     assert_eq!(
//     //         Metadata::try_from(&bad_field).unwrap_err().to_string(),
//     //         "External error: expected ident at line 1 column 2"
//     //     );
//     // }
// }
