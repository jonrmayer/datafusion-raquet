use geo::MakeValid;
use geo::Point;
use geo::Validation;
use geo::{BooleanOps, BoundingRect, Geometry};

use geo::{LineString, MultiPolygon, Polygon};

// use geo::Point;
use geo_types::point;

#[derive(Debug, Clone)]
pub struct Geo {
    pub geom: Geometry,
}

impl Geo {
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

    pub fn validate(&self) -> Geometry {
        let validated = match self.geom().is_valid() {
            true => self.geom(),
            false => {
                let (x, y) = self.min_xy();
                let p = point! {
                    x: x,
                    y: y,
                };

                // let Point(coord) = p;
                Geometry::Point(p)
            }
        };

        validated
    }

    pub fn to_polys(&self) -> Vec<Polygon> {
        let mut polys = Vec::new();

        match self.geom() {
            // Handle a single Polygon
            Geometry::Polygon(poly) => {
                polys.push(poly);
            }

            // Handle a MultiPolygon (collection of Polygons)
            Geometry::MultiPolygon(multi_poly) => {
                for poly in multi_poly {
                    polys.push(poly);
                }
            }

            _ => {}
        }
        polys
    }

    pub fn intersections(&self, tile_geoms: Vec<Polygon>) {
        let polys = self.to_polys();
        for tile_geom in tile_geoms {
            let intersection_shape: MultiPolygon<f64> = polys[0].intersection(&tile_geom);
        }
    }

    pub fn polygon_to_linestrings(&self) -> Vec<LineString<f64>> {
        let mut lines = Vec::new();

        match self.geom() {
            // Handle a single Polygon
            Geometry::Polygon(poly) => {
                let (exterior, interiors) = poly.into_inner();
                lines.push(exterior);
                lines.extend(interiors);
            }

            // Handle a MultiPolygon (collection of Polygons)
            Geometry::MultiPolygon(multi_poly) => {
                for poly in multi_poly {
                    let (exterior, interiors) = poly.into_inner();
                    lines.push(exterior);
                    lines.extend(interiors);
                }
            }

            _ => {}
        }

        lines
    }
}
