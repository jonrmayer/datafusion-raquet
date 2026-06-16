mod error;
mod proj;
mod rasterizer;
mod tiles;
mod transforms;
mod formats;

mod base;

use geo::map_coords::MapCoords;
use geo::{BoundingRect, Geometry};
use quadbin_rs::{
    Tile, lonlat_to_cell, lonlat_to_tile, tile_to_bbox_mercator, tile_to_bbox_wgs84, tile_to_cell,
};
use std::cmp;
use wkt::TryFromWkt;

use crate::tiles::GeoTiles;

use crate::base::BaseGeo;
pub use crate::formats::GeoFormats;

#[derive(Debug)]
pub struct GeoCells {
    pub geo: BaseGeo,
   
}

impl GeoCells {
    pub fn new(wkt: String, resolution: i8) -> Self {
        let geom = geo_types::Geometry::<f64>::try_from_wkt_str(&wkt).unwrap();
        let geo =BaseGeo::new(geom, resolution);
        Self { geo }
    }

    pub fn geo(&self) -> BaseGeo {
        self.geo.clone()
    }

   
    pub fn bounding_cells(&self) -> Vec<u64> {
        let mut result: Vec<u64> = Vec::new();
        let gt = GeoTiles::new(self.geo());
        let (min_x, min_y, max_x, max_y) = gt.tile_extent();

        for new_x in min_x..=max_x {
            for new_y in max_y..=min_y {
                let new_tile = Tile {
                    x: new_x,
                    y: new_y,
                    z: self.geo().resolution() as u8,
                };
                let cell = tile_to_cell(new_tile);
                result.push(cell);
            }
        }
        result
    }

    pub fn intersecting_cells(&self) -> Vec<u64> {
        let mut result: Vec<u64> = vec![];
        let geotiles = GeoTiles::new(self.geo());
        // let w_h = self.tile_width_height();
        // let max = w_h.0.max(w_h.1);
        // let base_resolution =  self.resolution()-max.ilog2() as i8;
        println!("{:?}  ",geotiles.geos(5));


        // let tile_geom = transforms::transform_latlon_to_tile_coord(self.geom(), self.resolution());

        // let georast = rasterizer::GeoRasterizer::new(tile_geom);

        // let pixels = georast.intersecting_pixels();
        // result = pixels
        //     .iter()
        //     .map(|p| {
        //         let new_tile = Tile {
        //             x: p.0 as u32,
        //             y: p.1 as u32,
        //             z: self.resolution() as u8,
        //         };
        //         let cell = tile_to_cell(new_tile);
        //         cell
        //     })
        //     .collect();

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use euclid::{Transform2D, UnknownUnit};
    // use base::map_coords::MapCoords;
    // use geo_rasterizer::{MergeAlgorithm, Rasterizer};
    // use vaster::*;

    // Define your custom units to keep your spaces separate

    // pub fn pix_to_geo_transform(gt: &[f64; 6]) -> Transform2D<f64, UnknownUnit, UnknownUnit> {
    //     Transform2D::new(
    //         gt[1], // m11
    //         gt[4], // m12
    //         gt[2], // m21
    //         gt[5], // m22
    //         gt[0], // m31
    //         gt[3], // m32
    //     )
    // }

    #[test]
    fn it_works() {
        let wkt_str = "POLYGON((-74.0 40.7, -73.9 40.7, -73.9 40.8, -74.0 40.8, -74.0 40.7))";
        let linestring_str = "LINESTRING(-45 40.979898069620134, 0 40.979898069620134, 0 66.51326044311186, -45 66.51326044311186, -45 40.979898069620134)";
        let wkt_pt_str = "POINT(-74.0 40.7)";
        let z: i8 = 25;
       

        let gc = GeoCells::new(linestring_str.to_string(), z);
        // println!("extent {:?}", gc.extent());
        // let bounding = gc.bounding_cells();
        // println!("bounding cells {:?}", bounding.len());
        let intersecting = gc.intersecting_cells();
        // println!("intersecting cells {:?}", intersecting.len());
    }
}
