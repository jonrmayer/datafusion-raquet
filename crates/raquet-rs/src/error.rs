use quadbin_rs::QuadBinError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RaquetError {
    #[error("Coordinate validation boundary error: longitude {lon}, latitude {lat} outside limits")]
    InvalidCoordinates { lon: f64, lat: f64 },
    #[error("Quadbin indexing resolution {0} is invalid (must be between 0 and 26)")]
    InvalidResolution(i8),
    #[error("Buffer mismatch: expected identical slice sizes but found lengths {0} and {1}")]
    BufferSizeMismatch(usize, usize),
    #[error("Underlying spatial tracking index initialization failed: {0:?}")]
    IndexInitializationError(QuadBinError),
    #[error("Failed to decode cell index to spatial coordinates: {0:?}")]
    CoordinateDecodingError(QuadBinError),
    #[error("Hierarchical operational error: {0:?}")]
    HierarchyError(QuadBinError),
}
