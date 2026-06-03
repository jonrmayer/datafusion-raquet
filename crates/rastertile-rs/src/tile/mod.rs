mod array;
mod data_type;

pub use data_type::DataType;

pub use array::{Array, TypedArray};


use crate::RasterTileResult;

use crate::{Compression, CompressionFormat};

use rolling_stats::Stats;

/// Statistics computed from a buffer
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileStatistics {
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Mean value
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Number of valid (non-nodata) pixels
    pub valid_count: u64,
}

/// A TIFF Tile response.
///
/// This contains the required information to decode the tile. Decoding is separated from fetching
/// so that sync and async operations can be separated and non-blocking.
///
/// This is returned by `fetch_tile`.
///
/// A strip of a stripped tiff is an image-width, rows-per-strip tile.
#[derive(Debug)]
pub struct Tile {
    pub x: usize,
    pub y: usize,
    pub data_type: Option<DataType>,

    pub compressed_bytes: Vec<u8>,
    pub compression_method: CompressionFormat,
}

impl Tile {
    /// The column index of this tile.
    pub fn x(&self) -> usize {
        self.x
    }

    /// The row index of this tile.
    pub fn y(&self) -> usize {
        self.y
    }

    /// Access the compressed bytes underlying this tile.
    ///
    /// Note that [`Bytes`] is reference-counted, so it is very cheap to clone if needed.
    pub fn compressed_bytes(&self) -> &Vec<u8> {
        &self.compressed_bytes
    }

    /// Access the compression tag representing this tile.
    pub fn compression_method(&self) -> CompressionFormat {
        self.compression_method
    }

    /// Decode this tile to an [`Array`].
    ///
    /// Decoding is separate from data fetching so that sync and async operations do not block the
    /// same runtime.
    pub fn decode(self) -> RasterTileResult<Array> {
        let compression = Compression {
            format: self.compression_method(),
        };
        let decoded = compression.decompress(self.compressed_bytes());

        let shape = [1, self.x(), self.y()];
        Array::try_new(decoded, shape, self.data_type)
    }

    pub fn statistics(self) -> RasterTileResult<TileStatistics> {
        let a = self.decode().unwrap();
        let bb = a.data().clone();
        let mut stats: Stats<f32> = Stats::new();
        match bb {
            TypedArray::Float32(v) => {
                v.iter().for_each(|v| stats.update(*v));
            }
            _ => panic!("expected Float32"),
        };
        let out = TileStatistics {
            min: stats.min as f64,
            max: stats.max as f64,
            mean: stats.mean as f64,
            std_dev: stats.std_dev as f64,
            valid_count: stats.count as u64,
        };
        Ok(out)
    }
}

// fn infer_shape(
//     planar_configuration: PlanarConfiguration,
//     width: usize,
//     height: usize,
//     samples_per_pixel: usize,
// ) -> [usize; 3] {
//     match planar_configuration {
//         PlanarConfiguration::Chunky => [height, width, samples_per_pixel],
//         PlanarConfiguration::Planar => [samples_per_pixel, height, width],
//     }
// }
