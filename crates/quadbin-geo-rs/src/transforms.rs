use geo::map_coords::MapCoords;
use geo::{BoundingRect, Geometry};
use quadbin_rs::{Tile, lonlat_to_tile, tile_to_bbox_mercator, tile_to_bbox_wgs84, tile_to_cell};

use crate::error::GeoError;
use crate::proj::{latlon_from_mercator, mercator_from_latlon};

pub fn transform_latlon_to_mercator(in_geom: Geometry) -> Geometry {
    let transform_mercator = |c: geo_types::Coord<f64>| -> Result<_, GeoError> {
        let (x, y) = mercator_from_latlon(c.x, c.y);
        let out = geo_types::Coord { x: x, y: y };
        Ok(out)
    };
    let out_geom = in_geom
        .try_map_coords(|coord| transform_mercator(coord))
        .unwrap();

    out_geom
}

pub fn transform_latlon_to_tile_coord(in_geom: Geometry, resolution: i8) -> Geometry {
    let transform_to_tile = |c: geo_types::Coord<f64>| -> Result<_, GeoError> {
        let tile = lonlat_to_tile(c.x, c.y, resolution);
        let out = geo_types::Coord {
            x: tile.x as f64,
            y: tile.y as f64,
        };
        Ok(out)
    };
    let out_geom = in_geom
        .try_map_coords(|coord| transform_to_tile(coord))
        .unwrap();

    out_geom
}

pub fn transform_tile_to_local_coord(in_geom: Geometry, min_x: f64, min_y: f64) -> Geometry {
        let transform_to_local = |c: geo_types::Coord<f64>| -> Result<_, GeoError> {
           
            let out = geo_types::Coord {
                x: c.x - min_x,
                y: c.y - min_y,
            };
            Ok(out)
        };
        let out_geom = 
            in_geom
            .try_map_coords(|coord| transform_to_local(coord))
            .unwrap();

        out_geom
    }
