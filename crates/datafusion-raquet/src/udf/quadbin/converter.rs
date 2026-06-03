use arrow_array::array::{Array, ArrayRef};
use arrow_schema::{DataType, Field};

use arrow_convert::{
    ArrowDeserialize, ArrowField, ArrowSerialize, deserialize::TryIntoCollection,
    serialize::TryIntoArrow,
};
use quadbin_rs::Bbox;

pub fn bbox_to_wkt(bbox: quadbin_rs::Bbox) -> String {
    let min_x = bbox.min_x;
    let min_y = bbox.min_y;
    let max_x = bbox.max_x;
    let max_y = bbox.max_y;
    let wkt = format!(
        "POLYGON(({min_x} {min_y},{max_x} {min_y},{max_x} {max_y},{min_x} {max_y},{min_x} {min_y}))"
    );
    wkt
}

pub fn bbox_to_geojson(bbox: quadbin_rs::Bbox) -> String {
    let min_x = bbox.min_x;
    let min_y = bbox.min_y;
    let max_x = bbox.max_x;
    let max_y = bbox.max_y;
    let json_start = r#"{"type":"Polygon","coordinates":[["#;
    let json_end = r#"]]}"#;

    let coords = format!(
        "[{min_x},{min_y}],[{max_x},{min_y}],[{max_x},{max_y}],[{min_x},{max_y}],[{min_x},{min_y}]"
    );
    let geojson = format!("{json_start}{coords}{json_end}");
    geojson
}

#[derive(Debug, Clone, PartialEq, ArrowField, ArrowSerialize, ArrowDeserialize)]
pub struct Abbox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl Abbox {
    pub fn new(bbox: quadbin_rs::Bbox) -> Self {
        Abbox {
            min_x: bbox.min_x,
            min_y: bbox.min_y,
            max_x: bbox.max_x,
            max_y: bbox.max_y,
        }
    }
}

#[derive(Debug, Clone, PartialEq, ArrowField, ArrowSerialize, ArrowDeserialize)]
pub struct LonLat {
    pub lon: f64,
    pub lat: f64,
}

impl LonLat {
    pub fn new(lon: f64, lat: f64) -> Self {
        LonLat { lon, lat }
    }
}

#[derive(Debug, Clone, PartialEq, ArrowField, ArrowSerialize, ArrowDeserialize)]
pub struct Pixel {
    pub pixel_x: i32,
    pub pixel_y: i32,
}

impl Pixel {
    pub fn new(pixel_x: i32, pixel_y: i32) -> Self {
        Pixel { pixel_x, pixel_y }
    }
}
