use thiserror::Error;

#[derive(Debug, Error)]
pub enum QuadBinError {
    // InvalidDirection(u8),
    // InvalidCell(Option<u64>),
    // InvalidResolution(u8),
    // InvalidOffset(f64),
    #[error("QuadBinError error: {0}")]
    General(String),
}

pub type QuadBinResult<T> = std::result::Result<T, QuadBinError>;
