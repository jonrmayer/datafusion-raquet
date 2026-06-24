use thiserror::Error;

/// Result type for compression operations
pub type OperationsResult<T> = std::result::Result<T, OperationsError>;

use crate::compression::CompressionError;
use crate::data_types::DataTypeError;

/// Errors that can occur during compression operations
#[derive(Error, Debug)]
pub enum OperationsError {
     /// Array error.
    #[error("Array error: {0}")]
    Array(String),

     #[error("NDArray error: {0}")]
    NDArray(String),

     #[error("General error: {0}")]
    General(String),

    #[error(transparent)]
    CompressionError(#[from] CompressionError),

    #[error(transparent)]
    DataTypeError(#[from] DataTypeError),


}

