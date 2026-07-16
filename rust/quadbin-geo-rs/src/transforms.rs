use geo::map_coords::MapCoords;
use geo::{ Geometry};
use quadbin_rs::{ Tile};

use crate::error::QuadBinGeoError;
use crate::proj::{ mercator_from_latlon};

#[allow(dead_code)]
pub fn transform_latlon_to_mercator(in_geom: Geometry) -> Geometry {
    let transform_mercator = |c: geo_types::Coord<f64>| -> Result<_, QuadBinGeoError> {
        let (x, y) = mercator_from_latlon(c.x, c.y);
        let out = geo_types::Coord { x, y };
        Ok(out)
    };
    

    in_geom
        .try_map_coords(transform_mercator)
        .unwrap()
}

pub fn transform_latlon_to_tile_coord(in_geom: Geometry, resolution: i8) -> Geometry {
    let transform_to_tile = |c: geo_types::Coord<f64>| -> Result<_, QuadBinGeoError> {
        let tile = Tile::from_lonlat(c.x, c.y, resolution).unwrap();
        let out = geo_types::Coord {
            x: tile.x as f64,
            y: tile.y as f64,
        };
        Ok(out)
    };
    

    in_geom
        .try_map_coords(transform_to_tile)
        .unwrap()
}

pub fn transform_tile_to_local_coord(in_geom: Geometry, min_x: f64, min_y: f64) -> Geometry {
    let transform_to_local = |c: geo_types::Coord<f64>| -> Result<_, QuadBinGeoError> {
        let out = geo_types::Coord {
            x: c.x - min_x,
            y: c.y - min_y,
        };
        Ok(out)
    };
    

    in_geom
        .try_map_coords(transform_to_local)
        .unwrap()
}
