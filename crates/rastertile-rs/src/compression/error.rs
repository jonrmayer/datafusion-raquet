use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("IO error encountered during compression/decompression: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization or byte-casting error: {0}")]
    ByteCastError(String),

    #[error("Gzip strategy failure: {0}")]
    GzipError(String),

    #[error("JPEG engine operational failure: {0}")]
    JpegError(String),

    #[error("WebP engine operational failure: {0}")]
    WebpError(String),

    #[error("Invalid data shape or array layout: {0}")]
    InvalidDimensions(String),
}
