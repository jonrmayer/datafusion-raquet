use geo::{BoundingRect, Geometry};

#[derive(Debug, Clone)]
pub struct BaseGeo {
    pub geom: Geometry,
    pub resolution: i8,
}

impl BaseGeo {
    pub fn new(geom: Geometry, resolution: i8) -> Self {
        Self { geom, resolution }
    }

    pub fn geom(&self) -> Geometry {
        self.geom.clone()
    }
    pub fn resolution(&self) -> i8 {
        self.resolution
    }
    pub fn extent(&self) -> (f64, f64, f64, f64) {
        let bounds = self.geom().bounding_rect().unwrap();
        let min_x = f64::min(bounds.min().x, bounds.max().x);
        let max_x = f64::max(bounds.min().x, bounds.max().x);

        let min_y = f64::min(bounds.min().y, bounds.max().y);
        let max_y = f64::max(bounds.min().y, bounds.max().y);
        (min_x, min_y, max_x, max_y)
    }
}
