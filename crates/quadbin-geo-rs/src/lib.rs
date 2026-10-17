mod error;
mod formats;
mod proj;
mod rasterizer;
mod tiles;
mod transforms;

mod base;
// mod masking;

mod geo_utils;
use geo_utils::Geo;

use geo::coord;
use geo::{Area, BooleanOps, BoundingRect};
use geo::{Geometry, Polygon, Rect};

use geo::{LineString, MultiPolygon};

use quadbin_rs::Tile;

use wkt::TryFromWkt;

use crate::tiles::GeoTiles;

use crate::base::BaseGeo;
pub use crate::formats::GeoFormats;

pub use error::{QuadBinGeoError, QuadBinGeoResult};

use std::collections::HashSet;

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

    pub fn contained_cells(&self) -> QuadBinGeoResult<Vec<u64>> {
        // let mut result: Vec<u64> = vec![];

        let intersecting_cells = self.intersecting_cells()?;
        let boundary_cells = self.boundary_cells()?;

        // 1. Collect boundary cells into a set for O(1) lookups
        let boundary_set: HashSet<_> = boundary_cells.into_iter().collect();

        // 2. Filter out anything that exists in the boundary set
        let result_cells: Vec<_> = intersecting_cells
            .into_iter()
            .filter(|cell| !boundary_set.contains(cell))
            .collect();
        Ok(result_cells)
    }

    pub fn boundary_cells(&self) -> QuadBinGeoResult<Vec<u64>> {
        let mut result: Vec<u64> = vec![];

        let tile_geom =
            transforms::transform_latlon_to_tile_coord(self.geo().geom(), self.geo().resolution());

        let linestrings = Geo::new(tile_geom).polygon_to_linestrings();
        for ls in linestrings {
            let geo = Geometry::LineString(ls);
            let georast = rasterizer::GeoRasterizer::new(geo);

            let intersecting_tiles: Vec<u64> = georast
                .intersecting()
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
            result.extend(intersecting_tiles);
        }

        Ok(result)
    }

    pub fn intersections(&self, tile_geoms: Vec<Polygon>) {
        let validated_geom = Geo::new(self.geo().geom());
        let polys = validated_geom.to_polys();

        for tile_geom in tile_geoms {
            let intersection_shape: MultiPolygon<f64> = polys[0].intersection(&tile_geom);
            let flat_area = intersection_shape.unsigned_area();
            println!("Flat Cartesian Area: {} square units", flat_area);
            // println!("{:?}",intersection_shape.);
        }
    }

    pub fn intersecting_cells(&self) -> QuadBinGeoResult<Vec<u64>> {
        let mut result: Vec<u64> = vec![];

        let tile_geom =
            transforms::transform_latlon_to_tile_coord(self.geo().geom(), self.geo().resolution());
        let validated_geom = Geo::new(tile_geom).validate();

        let pixels = match validated_geom {
            Geometry::Point(p) => Some(vec![(p.x(), p.y())]),
            Geometry::LineString(ls) => {
                let geo = Geometry::LineString(ls);
                let georast = rasterizer::GeoRasterizer::new(geo);

                let out = georast.intersecting();
                Some(out)
            }
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
    use quadbin_rs::QuadBin;

    #[test]
    fn test_tile_intersections() {
        let wkt_str = "POLYGON((-45 40.9798980696201, 0 40.9798980696201, 0 66.5132604431119, -45 66.5132604431119, -45 40.9798980696201))";

        let z: i8 = 9;

        let gc = GeoCells::new(wkt_str.to_string(), z);
        // println!("extent {:?}", gc.extent());
        let boundary_cells = gc.boundary_cells().unwrap();
        let mut tile_geoms: Vec<Polygon> = Vec::new();
        for boundary_cell in boundary_cells {
            let bbox = QuadBin::from_cell(boundary_cell)
                .unwrap()
                .to_tile()
                .unwrap()
                .to_bbox_wgs84()
                .unwrap();

            // 1. Define your bounding box corners (min x/y and max x/y)
            let min_coord = coord! { x: bbox.min_x, y:bbox.min_y }; // e.g., Southwest
            let max_coord = coord! { x: bbox.max_x, y:bbox.max_y }; // e.g., Northeast

            // 2. Create the Rect bounding box
            let bbox = Rect::new(min_coord, max_coord);

            // 3. Convert the Rect directly into a Polygon
            let polygon: Polygon<f64> = bbox.into();
            tile_geoms.push(polygon);
            // println!("{:?}", polygon);
        }
        gc.intersections(tile_geoms);
    }

    #[test]
    fn test_bounding_cells() {
        let wkt_str = "POLYGON((-45 40.9798980696201, 0 40.9798980696201, 0 66.5132604431119, -45 66.5132604431119, -45 40.9798980696201))";
        let _linestring_str = "LINESTRING(-45 40.979898069620134, 0 40.979898069620134, 0 66.51326044311186, -45 66.51326044311186, -45 40.979898069620134)";
        let _wkt_pt_str = "POINT(-74.0 40.7)";
        let z: i8 = 9;

        let gc = GeoCells::new(wkt_str.to_string(), z);
        // println!("extent {:?}", gc.extent());
        let bounding = gc.bounding_cells();
        println!("bounding cells {:?}", bounding.unwrap().len());
        // let intersecting = gc.intersecting_cells();
        // println!("intersecting cells {:?}", intersecting.len());
    }

    #[test]
    fn test_intersecting_cells() {
        let wkt_str = "POLYGON((-45 40.9798980696201, 0 40.9798980696201, 0 66.5132604431119, -45 66.5132604431119, -45 40.9798980696201))";
        let _linestring_str = "LINESTRING(-45 40.979898069620134, 0 40.979898069620134, 0 66.51326044311186, -45 66.51326044311186, -45 40.979898069620134)";
        let _wkt_pt_str = "POINT(-74.0 40.7)";
        let z: i8 = 9;

        let gc = GeoCells::new(wkt_str.to_string(), z);
        // println!("extent {:?}", gc.extent());
        let intersecting = gc.intersecting_cells();
        println!("intersecting_cells {:?}", intersecting.unwrap().len());
        // let intersecting = gc.intersecting_cells();
        // println!("intersecting cells {:?}", intersecting.len());
    }

    #[test]
    fn test_boundary_cells() {
        let wkt_str = "POLYGON((-45 40.9798980696201, 0 40.9798980696201, 0 66.5132604431119, -45 66.5132604431119, -45 40.9798980696201))";

        let z: i8 = 9;

        let gc = GeoCells::new(wkt_str.to_string(), z);
        // println!("extent {:?}", gc.extent());
        let boundary = gc.boundary_cells();
        println!("boundary_cells {:?}", boundary.unwrap().len());
        // let intersecting = gc.intersecting_cells();
        // println!("intersecting cells {:?}", intersecting.len());
    }

    #[test]
    fn test_contained_cells() {
        let wkt_str = "POLYGON((-45 40.9798980696201, 0 40.9798980696201, 0 66.5132604431119, -45 66.5132604431119, -45 40.9798980696201))";

        let z: i8 = 9;

        let gc = GeoCells::new(wkt_str.to_string(), z);
        // println!("extent {:?}", gc.extent());
        let contained = gc.contained_cells();
        println!("contained_cells {:?}", contained.unwrap().len());
        // let intersecting = gc.intersecting_cells();
        // println!("intersecting cells {:?}", intersecting.len());
    }
}
