use thiserror::Error;

/// Result type for compression operations
pub type DataTypeResult<T> = std::result::Result<T, DataTypeError>;

/// Errors that can occur during metadata operations
#[derive(Error, Debug)]
pub enum DataTypeError {
     /// DataType error.
    #[error("DataType error: {0}")]
    General(String),


}