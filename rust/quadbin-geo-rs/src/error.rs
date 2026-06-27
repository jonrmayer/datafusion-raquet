

use thiserror::Error;

#[derive(Debug, Error)]
pub enum QuadBinGeoError {
     #[error(transparent)]
    QuadBin(#[from] quadbin_rs::QuadBinError),


    #[error("QuadBinGeoError error: {0}")]
    General(String),
}



pub type QuadBinGeoResult<T> = std::result::Result<T, QuadBinGeoError>;