use geo::{Geometry, Rect, coord};

use quadbin_rs::Bbox;
use wkt::ToWkt;

#[derive(Debug)]
pub struct GeoFormats {
    pub bbox: Bbox,
}

impl GeoFormats {
    pub fn new(bbox: Bbox) -> Self {
        Self { bbox }
    }

    pub fn bbox(&self) -> Bbox {
        self.bbox.clone()
    }
    pub fn to_geometry(&self) -> Geometry {
        let rect = Rect::new(
            coord! { x  : self.bbox().min_x, y: self.bbox().min_y },
            coord! { x  : self.bbox().max_x, y: self.bbox().max_y },
        );
        let poly = rect.to_polygon();
        Geometry::Polygon(poly)
    }
    pub fn to_wkt(&self) -> String {
        let wkt = self.to_geometry().to_wkt();
        wkt.to_string()
    }
    pub fn to_geojson(&self) -> String {
        let gj = geojson::GeometryValue::from(&self.to_geometry());
        // gj.to_string()
        // let wkt = self.to_geometry().to_wkt();
        gj.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let bbox = Bbox {
            min_x: -45.0,
            min_y: 40.979898069620134,
            max_x: 0.0,
            max_y: 66.51326044311186,
        };
        let wkt = GeoFormats::new(bbox).to_geojson();
        println!("{:?}", wkt);
    }
}
