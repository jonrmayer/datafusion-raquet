//! Defines [`RaquetDataFusionError`], representing all errors returned by this crate.

use std::fmt::Debug;

use arrow_schema::ArrowError;
use datafusion::error::DataFusionError;
use thiserror::Error;

/// Enum with all errors in this crate.
#[derive(Error, Debug)]
pub(crate) enum RaquetDataFusionError {
    #[error(transparent)]
    Arrow(#[from] ArrowError),

    #[error(transparent)]
    DataFusion(#[from] DataFusionError),

    #[error(transparent)]
    QuadBin(#[from] quadbin_rs::QuadBinError),

    #[error(transparent)]
    QuadBinGeo(#[from] quadbin_geo_rs::QuadBinGeoError),

    #[error(transparent)]
    RasterTile(#[from] rastertile_rs::OperationsError),
}

/// Crate-specific result type.
pub(crate) type RaquetDataFusionResult<T> = std::result::Result<T, RaquetDataFusionError>;

impl From<RaquetDataFusionError> for DataFusionError {
    fn from(value: RaquetDataFusionError) -> Self {
        match value {
            RaquetDataFusionError::Arrow(err) => DataFusionError::ArrowError(Box::new(err), None),
            RaquetDataFusionError::DataFusion(err) => err,
            // GeoDataFusionError::GeoArrow(err) => DataFusionError::External(Box::new(err)),
            RaquetDataFusionError::QuadBin(err) => DataFusionError::External(Box::new(err)),
            RaquetDataFusionError::QuadBinGeo(err) => DataFusionError::External(Box::new(err)),
            RaquetDataFusionError::RasterTile(err) => DataFusionError::External(Box::new(err)),
        }
    }
}
