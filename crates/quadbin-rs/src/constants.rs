pub(crate) const MAX_RESOLUTION: u8 = 26;
pub(crate) const PI: f64 = 3.14159265358979323846;
pub(crate) const EARTH_RADIUS: f64 = 6378137.0;
pub(crate) const MAX_LATITUDE: f64 = 85.051128779806604;

pub(crate) const HEADER: u64 = 0x4000_0000_0000_0000;
pub(crate) const MODE: u64 = 0x0800_0000_0000_0000;
pub(crate) const FOOTER: u64 = 0x000F_FFFF_FFFF_FFFF;

// Magic numbers for Morton code interleaving
pub(crate) const B0: u64 = 0x5555_5555_5555_5555;
pub(crate) const B1: u64 = 0x3333_3333_3333_3333;
pub(crate) const B2: u64 = 0x0F0F_0F0F_0F0F_0F0F;

pub(crate) const B3: u64 = 0x00FF_00FF_00FF_00FF;
pub(crate) const B4: u64 = 0x0000_FFFF_0000_FFFF;
pub(crate) const B5: u64 = 0x0000_0000_FFFF_FFFF;
