use thiserror::Error;

/// Result type for compression operations
pub type MetadataResult<T> = std::result::Result<T, MetadataError>;

/// Errors that can occur during metadata operations
#[derive(Error, Debug)]
pub enum MetadataError {
    /// General error.
    #[error("Metadata error: {0}")]
    General(String),
}
