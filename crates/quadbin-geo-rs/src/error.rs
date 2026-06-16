use std::error::Error;
use std::fmt;


#[derive(Debug, PartialEq)]
pub enum GeoError {
    InvalidDirection(u8),
    InvalidCell(Option<u64>),
    InvalidResolution(u8),
    InvalidOffset(f64),
}

impl fmt::Display for GeoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeoError::InvalidDirection(e) => write!(f, "invalid direction: {}", e),
            GeoError::InvalidCell(e) => write!(f, "invalid cell index: {:?}", e),
            GeoError::InvalidResolution(e) => write!(
                f,
                "Invalid resolution specified: {}. Accepted values are between 0 and 26, inclusive",
                e
            ),
            GeoError::InvalidOffset(msg) => write!(f, "invalid offset: {}", msg),
        }
    }
}

impl Error for GeoError {}