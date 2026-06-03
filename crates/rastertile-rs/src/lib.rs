mod errors;
pub use errors::{RasterTileError,RasterTileResult};

mod compression;
pub use compression::Compression;
mod core;
mod metadata;
pub use metadata::{BinaryType, CompressionFormat, Metadata, RasterDataType};

pub use core::RasterDataType as CoreRasterDataType;
pub use core::{RasterBuffer,NoDataValue};

mod tile;
pub use tile::{Tile,DataType as NewDataType, TypedArray,TileStatistics};
// pub use types::*;

// mod reader;

// #[cfg(test)]
// mod tests {
//     // use super::*;
//     use arrow::util::pretty::print_batches;
//     use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
//     use parquet::errors::Result;
//     use serde_json::Value;
//     use std::fs::File;

//     #[test]
//     fn test_metadata() {
//         let path: String = "/home/jonrm/projects/pytesting/spain_solar_ghi.parquet".to_string();
//         let file = File::open(path).unwrap();

//         let parquet_reader = ParquetRecordBatchReaderBuilder::try_new(file)
//             .unwrap()
//             .with_batch_size(8192)
//             .build()
//             .unwrap();

//         let mut batches = Vec::new();

//         for batch in parquet_reader {
//             let b = batch.unwrap();
//             if b.num_rows() > 1 {
//                 batches.push(b);
//             }
//         }
//         // print_batches(&batches).unwrap();

//         // let feature: RaquetMetadata = serde_json::from_reader(file).unwrap();

//         // println!(" value {:?}", feature);
//     }

//     // #[test]
//     // fn test_colortable() {
//     //     let colortable_str = r#"{
//     //             "0": [
//     //                 0,
//     //                 0,
//     //                 0,
//     //                 255],
//     //                  "1": [
//     //                 0,
//     //                 0,
//     //                 0,
//     //                 255
//     //             ]
//     //                 }"#;
//     //     let v: Value = serde_json::from_str(colortable_str).unwrap();

//     //     let r: Vec<(String, [f64; 4])> = serde_json::from_value(v).unwrap();

//     //     println!(" value {:?}", r);
//     // }
// }
