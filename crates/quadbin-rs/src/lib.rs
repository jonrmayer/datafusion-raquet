pub(crate) const MAX_RESOLUTION: u8 = 26;
pub(crate) const PI: f64 = 3.141_592_653_589_793;
pub(crate) const EARTH_RADIUS: f64 = 6378137.0;
pub(crate) const MAX_LATITUDE: f64 = 85.051_128_779_806_6;

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

mod error;
pub use error::{QuadBinError, QuadBinResult};

pub struct QuadBin {
    pub cell: u64,
}

impl QuadBin {
    pub fn from_cell(cell: u64) -> QuadBinResult<QuadBin> {
        Ok(QuadBin { cell })
    }

    pub fn from_lonlat(lon: f64, lat: f64, z: i8) -> QuadBinResult<QuadBin> {
        let tile: Tile = Tile::from_lonlat(lon, lat, z)?;
        let cell = tile.to_cell()?;
        Ok(QuadBin { cell })
    }
    pub fn from_tile(tile: Tile) -> QuadBinResult<QuadBin> {
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
        let cell = HEADER | MODE | (z << 52) | ((ux | (uy << 1)) >> 12) | (FOOTER >> (z * 2));
        Ok(QuadBin { cell })
    }
    pub fn cell(&self) -> u64 {
        self.cell
    }
    pub fn resolution(&self) -> QuadBinResult<u8> {
        let out = ((self.cell() >> 52) & 0x1F) as u8;
        Ok(out)
    }
    pub fn to_tile(&self) -> QuadBinResult<Tile> {
        let z = self.resolution()?;
        let q = (self.cell() & FOOTER) << 12;
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
        let tile = Tile {
            x: ux as u32,
            y: uy as u32,
            z,
        };
        Ok(tile)
    }

    pub fn children(&self) -> QuadBinResult<Vec<u64>> {
        let children = self.children_resolution(self.resolution()? + 1)?;
        Ok(children)
    }

    // Get children cells at higher resolution (returns 4 children)
    pub fn children_resolution(&self, child_resolution: u8) -> QuadBinResult<Vec<u64>> {
        let mut result: Vec<u64> = vec![];
        let current_res = self.resolution()?;
        let _ = child_resolution <= current_res || child_resolution > MAX_RESOLUTION;
        let tile = self.to_tile()?;
        let res_diff = child_resolution - current_res;
        let children_per_dim = 1 << res_diff;
        let _count = children_per_dim * children_per_dim;
        let base_x = tile.x << res_diff;
        let base_y = tile.y << res_diff;

        for dy in 0..children_per_dim {
            for dx in 0..children_per_dim {
                let tile = Tile::from_xyz(base_x + dx, base_y + dy, child_resolution)?;
                let cell = tile.to_cell()?;
                result.push(cell);
            }
        }
        Ok(result)
    }

    pub fn parent_resolution(&self, parent_resolution: u8) -> QuadBinResult<u64> {
        let current_res = self.resolution()?;
        if parent_resolution > current_res {

            //  return Err(QuadbinError::InvalidResolution(z));
        }
        if parent_resolution == current_res {
            return Ok(self.cell());
        }

        let tile = self.to_tile()?;
        let shift = tile.z - parent_resolution;
        let parent_x = tile.x >> shift;
        let parent_y = tile.y >> shift;
        let new_tile: Tile = Tile {
            x: parent_x,
            y: parent_y,
            z: parent_resolution,
        };
        new_tile.to_cell()
    }

    // Get parent cell at resolution - 1
    pub fn parent(&self) -> QuadBinResult<u64> {
        let current_res = self.resolution()?;
        if current_res == 0 {
            return Ok(self.cell());
        }
        self.parent_resolution(current_res - 1)
    }

    // Convert quadbin cell to longitude/latitude (center of cell)
    pub fn to_lonlat(&self) -> QuadBinResult<(f64, f64)> {
        let tile = self.to_tile()?;
        let n: f64 = f64::powf(2.0, tile.z as f64);
        let lon = (tile.x as f64 + 0.5) / n * 360.0 - 180.0;
        let lat_rad = ((PI * (1.0 - 2.0 * (tile.y as f64 + 0.5) / n)).sinh()).atan();
        let lat = lat_rad * 180.0 / PI;
        Ok((lon, lat))
    }

    // Returns cells in a grid pattern around the center cell
    pub fn kring(&self, k: i32) -> QuadBinResult<Vec<u64>> {
        let mut result: Vec<u64> = vec![];
        // let _k < 0;
        let tile: Tile = self.to_tile()?;
        let max_coord = (1 << tile.z) - 1; // Maximum valid coordinate at this resolution
        let _diameter = 2 * k + 1;

        for dy in -k..=k {
            for dx in -k..=k {
                let nx = (tile.x as i32 + dx) as u32;
                let ny = (tile.y as i32 + dy) as u32;
                if nx > max_coord || ny > max_coord {
                    continue;
                }
                let tile = Tile::from_xyz(nx, ny, tile.z)?;
                let cell = tile.to_cell()?;
                result.push(cell);
            }
        }
        Ok(result)
    }

    // Get sibling cells (other children of the same parent)
    pub fn siblings(&self) -> QuadBinResult<Vec<u64>> {
        let parent = self.parent()?;
        let result = QuadBin::from_cell(parent)?.children()?;
        Ok(result)
    }
}

#[derive(Debug)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub z: u8,
}

impl Tile {
    pub fn from_xyz(x: u32, y: u32, z: u8) -> QuadBinResult<Tile> {
        Ok(Tile { x, y, z })
    }
    pub fn from_lonlat(lon: f64, mut lat: f64, z: i8) -> QuadBinResult<Tile> {
        if lat > MAX_LATITUDE {
            lat = MAX_LATITUDE;
        }
        if lat < -MAX_LATITUDE {
            lat = -MAX_LATITUDE;
        }

        let n: f64 = 2f64.powf(z as f64);

        let fx: f64 = ((lon + 180.0) / 360.0 * n).floor();
        let lat_rad = lat * PI / 180.0;

        let fy = ((1.0 - (lat_rad.tan().asinh()) / PI) / 2.0 * n).floor();

        let mut x: u32 = fx as u32;
        let mut y: u32 = fy as u32;
        let n_int: i8 = n as i8;

        // if x < 0 {
        //     x = 0;
        // }
        if fx >= n {
            x = (n_int - 1) as u32;
        }

        // if y < 0 {
        //     y = 0;
        // }
        if fy >= n {
            y = (n_int - 1) as u32;
        }
        Ok(Tile { x, y, z: z as u8 })
    }

    pub fn x(&self) -> u32 {
        self.x
    }
    pub fn y(&self) -> u32 {
        self.y
    }
    pub fn z(&self) -> u8 {
        self.z
    }
    pub fn is_valid(&self) -> QuadBinResult<bool> {
        if self.z() > MAX_RESOLUTION {
            Ok(false)
        } else {
            Ok(true)
        }
    }
    pub fn to_lonlat(&self) -> QuadBinResult<(f64, f64)> {
        let n: f64 = f64::powf(2.0, self.z() as f64);
        let lon = self.x() as f64 / n * 360.0 - 180.0;
        let lat_rad = ((PI * (1.0 - 2.0 * self.y() as f64 / n)).sinh()).atan();
        let lat = lat_rad * 180.0 / PI;
        Ok((lon, lat))
    }

    pub fn to_cell(&self) -> QuadBinResult<u64> {
        let z: u64 = self.z() as u64;
        let mut ux: u64 = (self.x() as u64) << (32 - z);
        let mut uy: u64 = (self.y() as u64) << (32 - z);
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

        let cell = HEADER | MODE | (z << 52) | ((ux | (uy << 1)) >> 12) | (FOOTER >> (z * 2));
        Ok(cell)
    }

    // Get tile bounds in Web Mercator (EPSG:3857)
    pub fn to_bbox_mercator(&self) -> QuadBinResult<Bbox> {
        let n: f64 = f64::powf(2.0, self.z() as f64);
        let tile_size = 2.0 * PI * EARTH_RADIUS / n;
        let min_x = self.x() as f64 * tile_size - PI * EARTH_RADIUS;
        let max_x = (self.x() as f64 + 1.0) * tile_size - PI * EARTH_RADIUS;
        let max_y = PI * EARTH_RADIUS - self.y() as f64 * tile_size;
        let min_y = PI * EARTH_RADIUS - (self.y() as f64 + 1.0) * tile_size;
        Ok(Bbox {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    // Get tile bounds in WGS84 (EPSG:4326)
    pub fn to_bbox_wgs84(&self) -> QuadBinResult<Bbox> {
        let n: f64 = f64::powf(2.0, self.z() as f64);
        let min_lon = self.x() as f64 / n * 360.0 - 180.0;
        let max_lon = (self.x() as f64 + 1.0) / n * 360.0 - 180.0;

        let min_lat_rad = (PI * (1.0 - 2.0 * (self.y() as f64 + 1.0) / n))
            .sinh()
            .atan();
        let max_lat_rad = (PI * (1.0 - 2.0 * self.y() as f64 / n)).sinh().atan();

        let min_lat = min_lat_rad * 180.0 / PI;
        let max_lat = max_lat_rad * 180.0 / PI;

        Ok(Bbox {
            min_x: min_lon,
            min_y: min_lat,
            max_x: max_lon,
            max_y: max_lat,
        })
    }
}

#[derive(Debug, Clone)]
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

        let resolution = QuadBin::from_cell(cell).unwrap().resolution().unwrap();
        println!("test_cell_to_resolution: {:?}", resolution);
    }

    #[test]
    fn test_cell_to_tile() {
        let cell: u64 = 5256690695657226239;
        let tile = QuadBin::from_cell(cell).unwrap().to_tile().unwrap();
        println!("test_cell_to_tile: {:?}", tile);
    }

    #[test]
    fn test_tile_to_cell() {
        let tile: Tile = Tile {
            x: 9664,
            y: 12333,
            z: 15,
        };
        let cell: u64 = tile.to_cell().unwrap();
        println!("test_tile_to_cell: {:?}", cell);
    }

    #[test]
    fn test_lonlat_to_tile() {
        let tile = Tile::from_lonlat(185.0, 85.0, 2).unwrap();
        println!("test_lonlat_to_tile: {:?}", tile);
    }

    #[test]
    fn test_lonlat_to_cell() {
        let cell = QuadBin::from_lonlat(-73.98, 40.75, 13).unwrap().cell();
        println!("test_lonlat_to_cell: {:?}", cell);
    }

    #[test]
    fn test_tile_to_lonlat() {
        let tile: Tile = Tile { x: 3, y: 0, z: 2 };

        let lonlat = tile.to_lonlat().unwrap();
        println!("tile_to_lonlat: {:?}", lonlat);
    }

    #[test]
    fn test_cell_to_lonlat() {
        let lonlat = QuadBin::from_cell(5256690695657226239)
            .unwrap()
            .to_lonlat()
            .unwrap();
        println!("cell_to_lonlat: {:?}", lonlat);
    }

    #[test]
    fn test_tile_to_bbox_mercator() {
        let tile: Tile = Tile { x: 3, y: 0, z: 2 };
        let bbox = tile.to_bbox_mercator().unwrap();
        println!("tile_to_bbox_mercator: {:?}", bbox);
    }

    #[test]
    fn test_tile_to_bbox_wgs84() {
        let tile: Tile = Tile { x: 3, y: 2, z: 3 };
        let bbox = tile.to_bbox_wgs84().unwrap();
        println!("tile_to_bbox_wgs84: {:?}", bbox);
    }

    #[test]
    fn test_cell_to_parent_resolution() {
        let parent_cell = QuadBin::from_cell(5256690695657226239)
            .unwrap()
            .parent_resolution(10)
            .unwrap();
        println!("cell_to_parent_resolution: {:?}", parent_cell);
    }

    #[test]
    fn test_cell_to_parent() {
        let parent_cell = QuadBin::from_cell(5256690695657226239)
            .unwrap()
            .parent()
            .unwrap();
        println!("cell_to_parent: {:?}", parent_cell);
    }

    #[test]
    fn test_cell_to_children_resolution() {
        let cells = QuadBin::from_cell(5256690695657226239)
            .unwrap()
            .children_resolution(16)
            .unwrap();
        println!("cell_to_children_resolution: {:?}", cells);
    }

    #[test]
    fn test_cell_to_children() {
        let cells = QuadBin::from_cell(5256690695657226239)
            .unwrap()
            .children()
            .unwrap();
        println!("cell_to_children: {:?}", cells);
    }

    #[test]
    fn test_cell_kring() {
        let cells = QuadBin::from_cell(5256690695657226239)
            .unwrap()
            .kring(3)
            .unwrap();
        println!("cell_kring: {:?}", cells);
    }

    #[test]
    fn test_cell_siblings() {
        let cells = QuadBin::from_cell(5256690695657226239)
            .unwrap()
            .siblings()
            .unwrap();
        println!("cell_siblings: {:?}", cells);
    }

    #[test]
    fn test_lonlat_to_pixel() {
        let cells = lonlat_to_pixel(0.0, 0.0, 4, 256);
        println!("lonlat_to_pixel: {:?}", cells);
    }
}
