use thiserror::Error;

/// Result type for compression operations
pub type CompressionResult<T> = std::result::Result<T, CompressionError>;

/// Errors that can occur during compression operations
#[derive(Error, Debug)]
pub enum CompressionError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Error while decoding JPEG data.
    #[error(transparent)]
    JPEGDecodingError(#[from] jpeg_decoder::Error),

    /// Error while decoding JPEG data.
    #[error(transparent)]
    WEBPDecodingError(#[from] image_webp::DecodingError),
}
