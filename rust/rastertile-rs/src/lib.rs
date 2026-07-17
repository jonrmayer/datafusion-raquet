mod error;
pub use error::{RasterTileError, RasterTileResult};

mod compression;
pub use compression::Compression;

mod data_types;

pub use data_types::DataType;
// mod core;
mod metadata;
pub use metadata::{BinaryType, CompressionFormat, Metadata};

// pub use core::RasterDataType as CoreRasterDataType;
// pub use core::{RasterBuffer,NoDataValue};

// mod tile;
// pub use tile::{Tile,DataType as NewDataType, TypedArray,TileStatistics};

mod operations;

pub use operations::Operations;
pub use operations::OperationsError;
