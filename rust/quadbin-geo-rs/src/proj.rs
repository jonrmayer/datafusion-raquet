
#[allow(dead_code)]
// length of semi-major axis of the WGS84 ellipsoid, i.e. radius at equator
const EARTH_RADIUS_KM: f64 = 6_378.137;
#[allow(dead_code)]
pub fn lon2x(lon: f64) -> f64 {
    EARTH_RADIUS_KM * 1000. * lon.to_radians()
}
#[allow(dead_code)]
pub fn x2lon(x: f64) -> f64 {
    (x / (EARTH_RADIUS_KM * 1000.)).to_degrees()
}
#[allow(dead_code)]
pub fn lat2y(lat: f64) -> f64 {
    ((lat.to_radians() / 2. + std::f64::consts::PI / 4.).tan()).log(std::f64::consts::E)
        * EARTH_RADIUS_KM
        * 1000.
}
#[allow(dead_code)]
pub fn y2lat(y: f64) -> f64 {
    (2. * ((y / (EARTH_RADIUS_KM * 1000.)).exp()).atan() - std::f64::consts::PI / 2.).to_degrees()
}
#[allow(dead_code)]
pub fn latlon_from_mercator(x: f64, y: f64) -> (f64, f64) {
    let lat = y2lat(y);
    let lon = x2lon(x);
    (lon, lat)
}

pub fn mercator_from_latlon(lon: f64, lat: f64) -> (f64, f64) {
    let x = lon2x(lon);
    let y = lat2y(lat);
    (x, y)
}

#[cfg(test)]
mod tests {
    use crate::proj::*;

    #[test]
    fn test_auspix_wgs84() {
        let merc = mercator_from_latlon(160.0, -25.727323225033057);
        println!("x: {:?},y: {:?}", merc.0,merc.1);

        let latlon = latlon_from_mercator(merc.0,merc.1);
        println!("x: {:?},y: {:?}", latlon.0,latlon.1);
    }

    #[test]
    fn test_wgs84_auspix() {}
}
