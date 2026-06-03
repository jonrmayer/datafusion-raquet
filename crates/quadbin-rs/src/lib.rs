pub(crate) const MAX_RESOLUTION: u8 = 26;
pub(crate) const PI: f64 = 3.14159265358979323846;
pub(crate) const EARTH_RADIUS: f64 = 6378137.0;
pub(crate) const MAX_LATITUDE: f64 = 85.051128779806604;

pub(crate) const HEADER: u64 = 0x4000_0000_0000_0000;
pub(crate) const MODE: u64 = 0x0800_0000_0000_0000;
pub(crate) const FOOTER: u64 = 0x000F_FFFF_FFFF_FFFF;

// Magic numbers for Morton code interleaving
pub(crate) const B: &[u64; 6] = &[
    0x5555_5555_5555_5555,
    0x3333_3333_3333_3333,
    0x0F0F_0F0F_0F0F_0F0F,
    0x00FF_00FF_00FF_00FF,
    0x0000_FFFF_0000_FFFF,
    0x0000_0000_FFFF_FFFF,
];

pub(crate) const S: &[u8; 5] = &[1, 2, 4, 8, 16];

use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum QuadbinError {
    InvalidDirection(u8),
    InvalidCell(Option<u64>),
    InvalidResolution(u8),
    InvalidOffset(f64),
}

impl fmt::Display for QuadbinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuadbinError::InvalidDirection(e) => write!(f, "invalid direction: {}", e),
            QuadbinError::InvalidCell(e) => write!(f, "invalid cell index: {:?}", e),
            QuadbinError::InvalidResolution(e) => write!(
                f,
                "Invalid resolution specified: {}. Accepted values are between 0 and 26, inclusive",
                e
            ),
            QuadbinError::InvalidOffset(msg) => write!(f, "invalid offset: {}", msg),
        }
    }
}

impl Error for QuadbinError {}

#[derive(Debug)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub z: u8,
}

#[derive(Debug)]
pub struct Bbox {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

#[derive(Debug)]
pub struct PixelCoord {
    pub pixel_x: f64,
    pub pixel_y: f64,
    pub tile_x: f64,
    pub tile_y: f64,
}

impl Tile {
    pub fn new(x: u32, y: u32, z: u8) -> Result<Tile, QuadbinError> {
        if z > MAX_RESOLUTION {
            return Err(QuadbinError::InvalidResolution(z));
        } else {
        }
        Ok(Tile { x, y, z })
    }
}
// Convert quadbin cell to tile zoom
pub fn cell_to_resolution(cell: u64) -> u8 {
    ((cell >> 52) & 0x1F) as u8
}
// Convert quadbin cell to tile coordinates
pub fn cell_to_tile(cell: u64) -> Tile {
    let z = cell_to_resolution(cell);
    let q = (cell & FOOTER) << 12;
    let mut ux = q;
    let mut uy = q >> 1;

    ux &= B[0];
    uy &= B[0];

    ux = (ux | (ux >> S[0])) & B[1];
    uy = (uy | (uy >> S[0])) & B[1];

    ux = (ux | (ux >> S[1])) & B[2];
    uy = (uy | (uy >> S[1])) & B[2];

    ux = (ux | (ux >> S[2])) & B[3];
    uy = (uy | (uy >> S[2])) & B[3];

    ux = (ux | (ux >> S[3])) & B[4];
    uy = (uy | (uy >> S[3])) & B[4];

    ux = (ux | (ux >> S[4])) & B[5];
    uy = (uy | (uy >> S[4])) & B[5];

    ux >>= 32 - z;
    uy >>= 32 - z;
    Tile {
        x: ux as u32,
        y: uy as u32,
        z: z,
    }
}

// Convert tile coordinates to quadbin cell
pub fn tile_to_cell(tile: Tile) -> u64 {
    let z: u64 = tile.z as u64;
    let mut ux: u64 = (tile.x as u64) << (32 - z);
    let mut uy: u64 = (tile.y as u64) << (32 - z);
    // Morton code interleaving (spread bits)
    ux = (ux | (ux << S[4])) & B[4];
    uy = (uy | (uy << S[4])) & B[4];

    ux = (ux | (ux << S[3])) & B[3];
    uy = (uy | (uy << S[3])) & B[3];

    ux = (ux | (ux << S[2])) & B[2];
    uy = (uy | (uy << S[2])) & B[2];

    ux = (ux | (ux << S[1])) & B[1];
    uy = (uy | (uy << S[1])) & B[1];

    ux = (ux | (ux << S[0])) & B[0];
    uy = (uy | (uy << S[0])) & B[0];

    // Combine: HEADER | MODE | resolution | interleaved_xy | trailing_ones
    HEADER | MODE | (z << 52) | ((ux | (uy << 1)) >> 12) | (FOOTER >> (z * 2))
}

pub fn lonlat_to_tile(lon: f64, mut lat: f64, z: i8) -> Tile {
    if lat > MAX_LATITUDE {
        lat = MAX_LATITUDE;
    }
    if lat < -MAX_LATITUDE {
        lat = -MAX_LATITUDE;
    }

    let n: f64 = f64::powf(2.0, z as f64);
    let fx: f64 = ((lon + 180.0) / 360.0 * n).floor();
    let lat_rad = lat * PI / 180.0;
    let fy: f64 = ((1.0 - (lat_rad.tan() / PI).asinh()) / PI) / 2.0 * n;

    let mut x: u32 = fx as u32;
    let mut y: u32 = fy as u32;
    let n_int: i8 = n as i8;

    if x < 0 {
        x = 0;
    }
    if fx >= n {
        x = (n_int - 1) as u32;
    }

    if y < 0 {
        y = 0;
    }
    if fy >= n {
        y = (n_int - 1) as u32;
    }
    Tile {
        x: x,
        y: y,
        z: z as u8,
    }
}

pub fn lonlat_to_cell(lon: f64, lat: f64, z: i8) -> u64 {
    let tile: Tile = lonlat_to_tile(lon, lat, z);
    tile_to_cell(tile)
}

pub fn tile_to_lonlat(tile: Tile) -> (f64, f64) {
    let n: f64 = f64::powf(2.0, tile.z as f64);
    let lon = tile.x as f64 / n * 360.0 - 180.0;
    let lat_rad = ((PI * (1.0 - 2.0 * tile.y as f64 / n)).sinh()).atan();
    let lat = lat_rad * 180.0 / PI;
    (lon, lat)
}

// Convert quadbin cell to longitude/latitude (center of cell)
pub fn cell_to_lonlat(cell: u64) -> (f64, f64) {
    let tile = cell_to_tile(cell);
    let n: f64 = f64::powf(2.0, tile.z as f64);
    let lon = (tile.x as f64 + 0.5) / n * 360.0 - 180.0;
    let lat_rad = ((PI * (1.0 - 2.0 * (tile.y as f64 + 0.5) / n)).sinh()).atan();
    let lat = lat_rad * 180.0 / PI;
    (lon, lat)
}

// Get tile bounds in Web Mercator (EPSG:3857)
pub fn tile_to_bbox_mercator(tile: Tile) -> Bbox {
    let n: f64 = f64::powf(2.0, tile.z as f64);
    let tile_size = 2.0 * PI * EARTH_RADIUS / n;
    let min_x = tile.x as f64 * tile_size - PI * EARTH_RADIUS;
    let max_x = (tile.x as f64 + 1.0) * tile_size - PI * EARTH_RADIUS;
    let max_y = PI * EARTH_RADIUS - tile.y as f64 * tile_size;
    let min_y = PI * EARTH_RADIUS - (tile.y as f64 + 1.0) * tile_size;
    Bbox {
        min_x,
        min_y,
        max_x,
        max_y,
    }
}

// Get tile bounds in WGS84 (EPSG:4326)
pub fn tile_to_bbox_wgs84(tile: Tile) -> Bbox {
    let n: f64 = f64::powf(2.0, tile.z as f64);
    let min_lon = tile.x as f64 / n * 360.0 - 180.0;
    let max_lon = (tile.x as f64 + 1.0) / n * 360.0 - 180.0;
    let min_lat_rad = ((PI * (1.0 - 2.0 * tile.y as f64 + 1.0 / n)).sinh()).atan();
    let max_lat_rad = ((PI * (1.0 - 2.0 * tile.y as f64 / n)).sinh()).atan();
    let min_lat = min_lat_rad * 180.0 / PI;
    let max_lat = max_lat_rad * 180.0 / PI;

    Bbox {
        min_x: min_lon,
        min_y: min_lat,
        max_x: max_lon,
        max_y: max_lat,
    }
}
// Get parent cell at lower resolution
pub fn cell_to_parent_resolution(cell: u64, parent_resolution: u8) -> u64 {
    let current_res = cell_to_resolution(cell);
    if parent_resolution > current_res {

        //  return Err(QuadbinError::InvalidResolution(z));
    }
    if parent_resolution == current_res {
        return cell;
    }

    let tile = cell_to_tile(cell);
    let shift = tile.z - parent_resolution;
    let parent_x = tile.x >> shift;
    let parent_y = tile.y >> shift;
    let tile: Tile = Tile {
        x: parent_x,
        y: parent_y,
        z: parent_resolution,
    };
    tile_to_cell(tile)
}

// Get parent cell at resolution - 1
pub fn cell_to_parent(cell: u64) -> u64 {
    let current_res = cell_to_resolution(cell);
    if current_res == 0 {
        return cell;
    }
    cell_to_parent_resolution(cell, current_res - 1)
}

// Get children cells at higher resolution (returns 4 children)
pub fn cell_to_children_resolution(cell: u64, child_resolution: u8) -> Vec<u64> {
    let mut result: Vec<u64> = vec![];
    let current_res = cell_to_resolution(cell);
    if child_resolution <= current_res || child_resolution > MAX_RESOLUTION {}
    let tile = cell_to_tile(cell);
    let res_diff = child_resolution - current_res;
    let children_per_dim = 1 << res_diff;
    let _count = children_per_dim * children_per_dim;
    let base_x = tile.x << res_diff;
    let base_y = tile.y << res_diff;

    for dy in 0..children_per_dim {
        for dx in 0..children_per_dim {
            let tile = Tile::new(base_x + dx, base_y + dy, child_resolution).unwrap();
            let cell = tile_to_cell(tile);
            result.push(cell);
        }
    }
    result
}

// Get immediate children (4 cells at resolution + 1)
pub fn cell_to_children(cell: u64) -> Vec<u64> {
    cell_to_children_resolution(cell, cell_to_resolution(cell) + 1)
}

// Get k-ring neighbors (cells within distance k)
// Returns cells in a grid pattern around the center cell
pub fn cell_kring(cell: u64, k: i32) -> Vec<u64> {
    let mut result: Vec<u64> = vec![];
    if k < 0 {}
    let tile: Tile = cell_to_tile(cell);
    let max_coord = (1 << tile.z) - 1; // Maximum valid coordinate at this resolution
    let _diameter = 2 * k + 1;

    for dy in -k..=k {
        for dx in -k..=k {
            let nx = (tile.x as i32 + dx) as u32;
            let ny = (tile.y as i32 + dy) as u32;
            if nx > max_coord || ny > max_coord {
                continue;
            }
            let tile = Tile::new(nx, ny, tile.z).unwrap();
            let cell = tile_to_cell(tile);
            result.push(cell);
        }
    }
    result
}

// Get sibling cells (other children of the same parent)
pub fn cell_siblings(cell: u64) -> Vec<u64> {
    let mut result: Vec<u64> = vec![cell, cell, cell, cell];

    let parent = cell_to_parent(cell);
    result = cell_to_children(parent);
    result
}
// Calculate pixel coordinates within a tile for a given lon/lat
pub fn lonlat_to_pixel(lon: f64, mut lat: f64, z: i8, tile_size: i16) -> PixelCoord {
    // Clamp latitude
    if lat > MAX_LATITUDE {
        lat = MAX_LATITUDE;
    }
    if lat < -MAX_LATITUDE {
        lat = -MAX_LATITUDE;
    }

    let n: f64 = f64::powf(2.0, z as f64);
    // Get fractional tile position
    let tile_x_frac = (lon + 180.0) / 360.0 * n;
    let lat_rad = lat * PI / 180.0;
    let tile_y_frac = (1.0 - lat_rad.tan().asinh() / PI) / 2.0 * n;
    // Integer tile coordinates
    let tile_x = tile_x_frac.floor();
    let tile_y = tile_y_frac.floor();
    // Pixel within tile
    let mut pixel_x = (tile_x_frac - tile_x) * tile_size as f64;
    let mut pixel_y = (tile_y_frac - tile_y) * tile_size as f64;
    // Clamp to valid range
    if pixel_x >= tile_size as f64 {
        pixel_x = tile_size as f64 - 1.0;
    }
    if pixel_y >= tile_size as f64 {
        pixel_y = tile_size as f64 - 1.0;
    }
    if pixel_x < 0.0 {
        pixel_x = 0.0;
    }
    if pixel_y < 0.0 {
        pixel_y = 0.0;
    }

    PixelCoord {
        pixel_x,
        pixel_y,
        tile_x,
        tile_y,
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_cell_to_resolution() {
        let cell: u64 = 5256690695657226239;

        let resolution = cell_to_resolution(cell);
        println!("test_cell_to_resolution: {:?}", resolution);
    }

    #[test]
    fn test_cell_to_tile() {
        let cell: u64 = 5256690695657226239;
        let tile = cell_to_tile(cell);
        println!("test_cell_to_tile: {:?}", tile);
    }

    #[test]
    fn test_tile_to_cell() {
        let tile: Tile = Tile {
            x: 9664,
            y: 12333,
            z: 15,
        };
        let cell: u64 = tile_to_cell(tile);
        println!("test_tile_to_cell: {:?}", cell);
    }

    #[test]
    fn test_lonlat_to_tile() {
        let tile = lonlat_to_tile(185.0, 85.0, 2);
        println!("test_lonlat_to_tile: {:?}", tile);
    }

    #[test]
    fn test_lonlat_to_cell() {
        let cell = lonlat_to_cell(-73.98, 40.75, 13);
        println!("test_lonlat_to_cell: {:?}", cell);
    }

    #[test]
    fn test_tile_to_lonlat() {
        let tile: Tile = Tile { x: 3, y: 0, z: 2 };

        let lonlat = tile_to_lonlat(tile);
        println!("tile_to_lonlat: {:?}", lonlat);
    }

    #[test]
    fn test_cell_to_lonlat() {
        let lonlat = cell_to_lonlat(5256690695657226239);
        println!("cell_to_lonlat: {:?}", lonlat);
    }

    #[test]
    fn test_tile_to_bbox_mercator() {
        let tile: Tile = Tile { x: 3, y: 0, z: 2 };
        let bbox = tile_to_bbox_mercator(tile);
        println!("tile_to_bbox_mercator: {:?}", bbox);
    }

    #[test]
    fn test_tile_to_bbox_wgs84() {
        let tile: Tile = Tile { x: 3, y: 0, z: 2 };
        let bbox = tile_to_bbox_wgs84(tile);
        println!("tile_to_bbox_wgs84: {:?}", bbox);
    }

    #[test]
    fn test_cell_to_parent_resolution() {
        let parent_cell = cell_to_parent_resolution(5256690695657226239, 10);
        println!("cell_to_parent_resolution: {:?}", parent_cell);
    }

    #[test]
    fn test_cell_to_parent() {
        let parent_cell = cell_to_parent(5256690695657226239);
        println!("cell_to_parent: {:?}", parent_cell);
    }

    #[test]
    fn test_cell_to_children_resolution() {
        let cells = cell_to_children_resolution(5256690695657226239, 16);
        println!("cell_to_children_resolution: {:?}", cells);
    }

    #[test]
    fn test_cell_to_children() {
        let cells = cell_to_children(5256690695657226239);
        println!("cell_to_children: {:?}", cells);
    }

    #[test]
    fn test_cell_kring() {
        let cells = cell_kring(5256690695657226239, 3);
        println!("cell_kring: {:?}", cells);
    }

    #[test]
    fn test_cell_siblings() {
        let cells = cell_siblings(5256690695657226239);
        println!("cell_siblings: {:?}", cells);
    }

    #[test]
    fn test_lonlat_to_pixel() {
        let cells = lonlat_to_pixel(0.0, 0.0, 4, 256);
        println!("lonlat_to_pixel: {:?}", cells);
    }

}
