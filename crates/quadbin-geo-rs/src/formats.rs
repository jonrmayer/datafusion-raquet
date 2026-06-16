use geo::{BoundingRect, Geometry, Rect, coord};
use wkt::ToWkt;
use geojson::GeoJson;
use quadbin_rs::Bbox;

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
            coord! { x  : self.bbox().min_y, y: self.bbox().min_x },
            coord! { x  : self.bbox().max_y, y: self.bbox().max_x },
        );
        let poly = rect.to_polygon();
        Geometry::Polygon(poly)
        
    }
    pub fn to_wkt(&self) -> String {
        let wkt = self.to_geometry().to_wkt();
        wkt.to_string()
    }
     pub fn to_geojson(&self) -> String {
        let gj =geojson::GeometryValue::from(&self.to_geometry());
        // gj.to_string()
        // let wkt = self.to_geometry().to_wkt();
        gj.to_string()
    }
}
