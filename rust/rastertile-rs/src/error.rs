// //! Module for all GeoJSON-related errors
// // use crate::Feature;
// // use serde_json::value::Value;
// use thiserror::Error;

// use std::num::ParseFloatError;

// /// Errors which can occur when encoding, decoding, and converting GeoJSON
// #[derive(Error, Debug)]
// pub enum Error {
//     #[error("IO Error: {0}")]
//     Io(std::io::Error),
//     // #[error("Error while deserializing JSON: {0}")]
//     // MalformedJson(serde_json::Error),

//     #[error("Encountered an unknown 'geometry' object type: `{0}`")]
//     GeometryUnknownType(String),

//     // #[error("Encountered a non-object type for GeoJSON: `{0}`")]
//     // GeoJsonExpectedObject(Value),

//     // #[error(
//     //     "Encountered neither object type nor null type for 'geometry' field on 'feature' object: `{0}`"
//     // )]
//     // FeatureInvalidGeometryValue(Value),

//     #[error("Expected a Feature mapping, but got a `{0}`")]
//     NotAFeature(String),

//     #[error("Expected GeoJSON type `{expected}`, found `{actual}`")]
//     ExpectedType { expected: String, actual: String },
//     // #[error("Expected a String value, but got a `{0}`")]
//     // ExpectedStringValue(Value),
//     #[error("Expected a GeoJSON property for `{0}`, but got None")]
//     ExpectedProperty(String),
//     #[error("Expected a floating-point value, but got None")]
//     ExpectedF64Value,
//     #[error("Expected an Array value, but got `{0}`")]
//     ExpectedArrayValue(String),
//     // #[error("Expected an owned Object, but got `{0}`")]
//     // ExpectedObjectValue(Value),

//     #[error("Unsupported ZoneId format '{0}'")]
//     UnsupportedZoneIdFormat(String),

//     #[error("Invalid hex ZoneId: '{0}'")]
//     InvalidHexId(String),

//     // Parsing primitives
//     #[error("Float parse error: {0}")]
//     Float(#[from] ParseFloatError),

//     #[error("Expected a Feature, FeatureCollection, or Geometry, but got an empty type")]
//     EmptyType,

//      #[error("invalid writer state: {0}")]
//     InvalidWriterState(&'static str),
// }

// pub type Result<T> = std::result::Result<T, Error>;

// // impl From<serde_json::Error> for Error {
// //     fn from(error: serde_json::Error) -> Self {
// //         Self::MalformedJson(error)
// //     }
// // }

// impl From<std::io::Error> for Error {
//     fn from(error: std::io::Error) -> Self {
//         Self::Io(error)
//     }
// }

use thiserror::Error;

use crate::metadata::MetadataError;
use crate::operations::OperationsError;

#[derive(Error, Debug)]
pub enum RasterTileError {
    // /// An error during De/Compression.
    // #[error(transparent)]
    // CompressionError(#[from] CompressionError),
    /// An error during Operations.
    #[error(transparent)]
    OperationsError(#[from] OperationsError),

    /// An error during Metadata.
    #[error(transparent)]
    MetadataError(#[from] MetadataError),

    /// General error.
    #[error("General error: {0}")]
    General(String),
}

// impl From<CompressionError> for OperationsError {
//     fn from(err: CompressionError) -> OperationsError {
//         OperationsError::UnsupportedError(err)
//     }
// }

pub type RasterTileResult<T> = std::result::Result<T, RasterTileError>;
