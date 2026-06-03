mod error;
use error::{MetadataError, Result};

// use parquet::errors::Result;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ColorInterp {
    #[default]
    Undefined,
    Gray,
    Palette,
    Red,
    Green,
    Blue,
    Alpha,
//  Extended GDAL 3.5+
    Pan,
    Coastal,
    Rededge,
    Nir,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum NoData {
    String(String),
    Float(f64),
    Int(i64),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BandInfo {
    pub name: Option<String>,
    pub description: Option<String>,
    pub r#type: Option<String>,

    pub nodata: Option<NoData>,

    pub unit: Option<String>,
    pub scale: Option<f64>,
    pub offset: Option<f64>,
    // pub colorinterp: Option<ColorInterp>,
    pub colortable: Option<Value>,

    pub has_scale: Option<f64>,
    pub has_offset: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tiling {
    pub scheme: Option<String>,
    pub block_width: Option<i32>,
    pub block_height: Option<i32>,
    pub min_zoom: Option<i32>,
    pub max_zoom: Option<i32>,
    pub pixel_zoom: Option<i32>,
    pub num_blocks: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Time {
    #[serde(rename = "cf:units")]
    pub units: Option<String>,
    #[serde(rename = "cf:calendar")]
    pub calendar: Option<String>,
    pub resolution: Option<String>,
    pub interpretation: Option<String>,
    pub count: Option<i32>,
    pub range: Option<Vec<i32>>,
}

#[derive(Debug, Serialize, Deserialize,Clone)]
pub struct RaquetFormat {
    pub file_format: Option<String>,
    pub version: Option<String>,

    pub width: Option<i32>,
    pub height: Option<i32>,

    pub crs: Option<String>,
    pub bounds: Option<[f64; 4]>,
    pub bounds_crs: Option<String>,

    pub compression: Option<String>,
    pub compression_quality: Option<i32>,
    pub band_layout: Option<String>,
    pub tiling: Option<Tiling>,

    pub tile_statistics: Option<bool>,
    pub tile_statistics_columns: Option<Vec<String>>,
    pub bands: Option<Vec<BandInfo>>,

    pub time: Option<Time>,
}

impl RaquetFormat {
    pub fn version(&self) -> Result<String> {
        match &self.version {
            Some(v) => Ok(v.clone()),
            None => Err(MetadataError::InvalidMetadata("version".to_string())),
        }
    }
    pub fn bands(&self) -> Result<Vec<BandInfo>> {
        match &self.bands {
            Some(bi) => Ok(bi.clone()),
            None => Err(MetadataError::InvalidMetadata("band_info".to_string())),
        }
    }

    pub fn tiling(&self) -> Result<Tiling> {
        match &self.tiling {
            Some(t) => Ok(t.clone()),
            None => Err(MetadataError::InvalidMetadata("tiling".to_string())),
        }
    }

    pub fn compression(&self) -> Result<String> {
        match &self.compression {
            Some(c) => Ok(c.clone()),
            None => Err(MetadataError::InvalidMetadata("compression".to_string())),
        }
    }
}

impl RaquetFormat {
    pub fn get_band_type(&self, band_index: usize) -> BandInfo {
        let bands = self.bands().unwrap().clone();
        // if band_index < 0 || band_index >= self.band_info().len() {}

        bands[band_index].clone()
    }

    pub fn get_compression(&self) -> String {
        let compression = self.compression().unwrap().clone();
        // if band_index < 0 || band_index >= self.band_info().len() {}

        compression
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::fs::File;

    #[test]
    fn test_metadata() {
        let file =
            File::open("/home/jonrm/projects/git/rastertile-rs/src/metadata/example_metadata.json")
                .unwrap();
        let feature: RaquetFormat = serde_json::from_reader(file).unwrap();

        println!(" value {:?}", feature);
    }

    #[test]
    fn test_colortable() {
        let colortable_str = r#"{
                "0": [
                    0,
                    0,
                    0,
                    255],
                     "1": [
                    0,
                    0,
                    0,
                    255
                ]
                    }"#;
        let v: Value = serde_json::from_str(colortable_str).unwrap();

        let r: Vec<(String, [f64; 4])> = serde_json::from_value(v).unwrap();

        println!(" value {:?}", r);
    }
}
