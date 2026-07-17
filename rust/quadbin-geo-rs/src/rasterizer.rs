use crate::transforms::transform_tile_to_local_coord;
// use euclid::{Transform2D, UnknownUnit};

use geo::{BoundingRect, Geometry};
use geo_rasterizer::{MergeAlgorithm, Rasterizer};
// use vaster::*;
#[derive(Debug)]
pub struct GeoRasterizer {
    pub geom: Geometry,
}
impl GeoRasterizer {
    pub fn new(geom: Geometry) -> Self {
        Self { geom }
    }

    pub fn geom(&self) -> Geometry {
        self.geom.clone()
    }

    pub fn extent(&self) -> (f64, f64, f64, f64) {
        let bounds = self.geom().bounding_rect().unwrap();
        let min_x = f64::min(bounds.min().x, bounds.max().x);
        let max_x = f64::max(bounds.min().x, bounds.max().x);

        let min_y = f64::min(bounds.min().y, bounds.max().y);
        let max_y = f64::max(bounds.min().y, bounds.max().y);
        (min_x, min_y, max_x, max_y)
    }

    pub fn width_height(&self) -> (f64, f64) {
        let (min_x, min_y, max_x, max_y) = self.extent();
        let width = max_x - min_x;
        let height = max_y - min_y;

        (width, height)
    }

    pub fn min_xy(&self) -> (f64, f64) {
        let (min_x, min_y, _max_x, _max_y) = self.extent();

        (min_x, min_y)
    }

    pub fn local_geom(&self) -> Geometry {
        let (min_x, min_y) = self.min_xy();

        transform_tile_to_local_coord(self.geom(), min_x, min_y)
    }

    pub fn intersecting(&self) -> Vec<(f64, f64)> {
        let local = self.local_geom();
        let (width, height) = self.width_height();
        let (min_x, min_y) = self.min_xy();
        let mut r = Rasterizer::new(
            width as usize,
            height as usize,
            None,
            MergeAlgorithm::Replace,
            0u8,
        );

        let _ = r.rasterize(&local, 1u8);
        let pixels_array = r.finish();
        let pixels_vec = pixels_array.clone().into_raw_vec_and_offset().0;

        let mut intersecting: Vec<(f64, f64)> = vec![];
        for (i, v) in pixels_vec.iter().enumerate() {
            if v == &1 {
                let x = min_x + (i % width as usize) as f64;
                let y = min_y + (i / width as usize) as f64;
                intersecting.push((x, y));
            }
        }
        intersecting
    }
}
