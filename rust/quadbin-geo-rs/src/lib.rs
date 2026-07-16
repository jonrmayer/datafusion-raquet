mod error;
mod formats;
mod proj;
mod rasterizer;
mod tiles;
mod transforms;

mod base;

use geo::Geometry;
use quadbin_rs::Tile;

use wkt::TryFromWkt;

use crate::tiles::GeoTiles;

use crate::base::BaseGeo;
pub use crate::formats::GeoFormats;

pub use error::{QuadBinGeoError, QuadBinGeoResult};

pub fn wkt_to_lonlat(wkt: String) -> (f64, f64) {
    let geom = geo_types::Geometry::<f64>::try_from_wkt_str(&wkt).unwrap();
    let lonlat = match geom {
        Geometry::Point(p) => Some((p.x(), p.y())),
        _ => None,
    };
    lonlat.unwrap()
}

#[derive(Debug)]
pub struct GeoCells {
    pub geo: BaseGeo,
}

impl GeoCells {
    pub fn new(wkt: String, resolution: i8) -> Self {
        let geom = geo_types::Geometry::<f64>::try_from_wkt_str(&wkt).unwrap();
        let geo = BaseGeo::new(geom, resolution);
        Self { geo }
    }

    pub fn geo(&self) -> BaseGeo {
        self.geo.clone()
    }

    pub fn lonlat(&self) -> (f64, f64) {
        let out = match self.geo().geom() {
            Geometry::Point(p) => Some((p.x(), p.y())),
            _ => None,
        };

        out.unwrap()
    }

    pub fn bounding_cells(&self) -> QuadBinGeoResult<Vec<u64>> {
        let mut result: Vec<u64> = Vec::new();
        let gt = GeoTiles::new(self.geo());
        let (min_x, min_y, max_x, max_y) = gt.tile_extent();

        for new_x in min_x..=max_x {
            for new_y in min_y..=max_y {
                let new_tile = Tile {
                    x: new_x,
                    y: new_y,
                    z: self.geo().resolution() as u8,
                };
                let cell = new_tile.to_cell()?;
                result.push(cell);
            }
        }
        Ok(result)
    }

    pub fn intersecting_cells(&self) -> QuadBinGeoResult<Vec<u64>> {
        let mut result: Vec<u64> = vec![];

        let tile_geom =
            transforms::transform_latlon_to_tile_coord(self.geo().geom(), self.geo().resolution());

        let pixels = match tile_geom {
            Geometry::Point(p) => Some(vec![(p.x(), p.y())]),
            Geometry::Polygon(poly) => {
                let geo = Geometry::Polygon(poly);
                let georast = rasterizer::GeoRasterizer::new(geo);

                let out = georast.intersecting();
                Some(out)
            }

            _ => None,
        };

        result = pixels
            .unwrap()
            .iter()
            .map(|p| {
                let new_tile = Tile {
                    x: p.0 as u32,
                    y: p.1 as u32,
                    z: self.geo().resolution() as u8,
                };

                new_tile.to_cell().unwrap()
            })
            .collect();

        Ok(result)
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
        let wkt_str = "POLYGON((-45 40.9798980696201, 0 40.9798980696201, 0 66.5132604431119, -45 66.5132604431119, -45 40.9798980696201))";
        let _linestring_str = "LINESTRING(-45 40.979898069620134, 0 40.979898069620134, 0 66.51326044311186, -45 66.51326044311186, -45 40.979898069620134)";
        let _wkt_pt_str = "POINT(-74.0 40.7)";
        let z: i8 = 5;

        let gc = GeoCells::new(wkt_str.to_string(), z);
        // println!("extent {:?}", gc.extent());
        let bounding = gc.intersecting_cells();
        println!("bounding cells {:?}", bounding.unwrap().len());
        // let intersecting = gc.intersecting_cells();
        // println!("intersecting cells {:?}", intersecting.len());
    }
}
