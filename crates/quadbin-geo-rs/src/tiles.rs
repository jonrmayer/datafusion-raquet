use geo::{BoundingRect, Geometry};
use quadbin_rs::{QuadBin, Tile};

use crate::rasterizer::GeoRasterizer;
use crate::transforms::transform_latlon_to_tile_coord;

use crate::BaseGeo;

#[derive(Debug)]
pub struct GeoTiles {
    pub geo: BaseGeo,
    // pub increment: i8,
}

impl GeoTiles {
    pub fn new(geo: BaseGeo) -> Self {
        Self { geo }
    }

    pub fn geo(&self) -> BaseGeo {
        self.geo.clone()
    }

    pub fn tile_extent(&self) -> (u32, u32, u32, u32) {
        let (min_x, min_y, max_x, max_y) = self.geo().extent();

        let min_tile = Tile::from_lonlat(min_x, min_y, self.geo().resolution()).unwrap();

        let max_tile = Tile::from_lonlat(max_x, max_y, self.geo().resolution()).unwrap();
        (min_tile.x, max_tile.y, max_tile.x, min_tile.y)
    }

    pub fn tile_width_height(&self) -> (u32, u32) {
        let (min_x, min_y, max_x, max_y) = self.tile_extent();
        let width = max_x - min_x;
        let height = max_y - min_y;

        (width, height)
    }

    pub fn geos(&self, increment: i8) {
        let resolutions = self.resolutions(increment);
        let tile_geom0 = transform_latlon_to_tile_coord(self.geo().geom(), resolutions[0]);
        let tile_geom1 = transform_latlon_to_tile_coord(self.geo().geom(), resolutions[1]);
        let gr0 = GeoRasterizer::new(tile_geom0);
        let gr1 = GeoRasterizer::new(tile_geom1);
        // gr.
        for (i, v) in gr0.intersecting().iter().enumerate() {
            println!("{:?}", 2.0_f64.powf(5.0) + v.0);
        }

        for (i, v) in gr1.intersecting().iter().enumerate() {
            println!("{:?}", v);
        }

        // let out: Vec<BaseGeo> = resolutions
        //     .iter()
        //     .map(|res| {
        //         let tile_geom = transform_latlon_to_tile_coord(self.geo().geom(), res.clone());
        //         BaseGeo::new(tile_geom, res.clone())
        //     })
        //     .collect();
        // out
    }

    pub fn resolutions(&self, increment: i8) -> Vec<i8> {
        let mut output: Vec<i8> = vec![];
        let w_h = self.tile_width_height();
        let max = w_h.0.max(w_h.1);
        let range = max.ilog2() as i8;
        let mut steps: i8 = (range / increment) + 1;

        for i in 0..steps {
            let res = self.geo().resolution() - (i * increment);
            output.push(res);
        }

        output.iter().copied().rev().collect()
    }
}
