//! Defines GeoArrow CRS metadata and CRS transforms used for writing GeoArrow data to file formats
//! that require different CRS representations.

use std::fmt::Debug;

use serde::{Deserialize, Serialize, de::value};
use serde_json::Value;

// use crate::ellipsoid::Ellipsoid;

use crate::error::{RasterArrowError, RasterArrowResult};

/// Coordinate Reference System information.
///
/// As of GeoArrow version 0.2, GeoArrow supports various CRS representations:
///
/// - A JSON object describing the coordinate reference system (CRS)
///   using [PROJJSON](https://proj.org/specifications/projjson.html).
/// - A string containing a serialized CRS representation. This option
///   is intended as a fallback for producers (e.g., database drivers or
///   file readers) that are provided a CRS in some form but do not have the
///   means to convert it to PROJJSON.
/// - Omitted, indicating that the producer does not have any information about
///   the CRS.
///
/// For maximum compatibility, producers should write PROJJSON.
///
/// Note that regardless of the axis order specified by the CRS, axis order will be interpreted
/// according to the wording in the [GeoPackage WKB binary
/// encoding](https://www.geopackage.org/spec130/index.html#gpb_format): axis order is always
/// (longitude, latitude) and (easting, northing) regardless of the the axis order encoded in the
/// CRS specification.
///
/// Note that [`PartialEq`] and [`Eq`] currently use their default, derived implementations, so
/// only `Crs` that are structurally exactly equal will compare as equal. Two different
/// representations of the same logical CRS will not compare as equal.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Raquet {
    pub raquet: Option<Value>,
}

impl Raquet {
    pub fn raquet_value(&self) -> Option<&Value> {
        self.raquet.as_ref()
    }
    pub fn from_json_value(value: Value) -> Self {
        let raquet = Some(value);
        Self { raquet: raquet }
    }

    pub fn from_str(value: &str) -> Self {
        let val: Value = serde_json::from_str(value).unwrap();
        Raquet::from_json_value(val)
    }

    pub fn to_str(&self) -> String {
        let val = self.raquet.clone().unwrap();
        let val_str: String = serde_json::to_string(&val).unwrap();
        val_str
    }

    /// Return `true` if we should include a CRS key in the GeoArrow metadata
    pub(crate) fn should_serialize(&self) -> bool {
        self.raquet.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unescape(s: &str) -> serde_json::Result<String> {
        serde_json::from_str(&format!("\"{}\"", s))
    }

    #[test]
    fn raquet_test() {
        let raquet_str = r#"{
    "file_format": "raquet",
    "version": "0.5.0",
    "width": 9216,
    "height": 7936,
    "crs": "EPSG:3857",
    "bounds": [-19.69, 26.43, 5.63, 44.09],
    "bounds_crs": "EPSG:4326",
    "compression": "gzip",
    "tiling": {
        "scheme": "quadbin",
        "block_width": 256,
        "block_height": 256,
        "min_zoom": 3,
        "max_zoom": 9,
        "pixel_zoom": 17,
        "num_blocks": 1116
    },
    "bands": [{
        "name": "band_1",
        "description": "Global Horizontal Irradiation",
        "type": "float32",
        "nodata": null,
        "unit": "kWh/m²/day",
        "colorinterp": "undefined",
        "STATISTICS_MINIMUM": 0.0,
        "STATISTICS_MAXIMUM": 6.42,
        "STATISTICS_MEAN": 0.67,
        "STATISTICS_STDDEV": 1.63,
        "STATISTICS_VALID_PERCENT": 100.0,
        "histogram": {
            "min": -0.01,
            "max": 6.17,
            "buckets": 256,
            "counts": [55644410, 0, "..."]
        }
    }]
}"#;

        let value: Value = serde_json::from_str(raquet_str).unwrap();       
        let raquet: Raquet = Raquet::from_json_value(value);
        println!("{:?}", raquet)
    }
}
