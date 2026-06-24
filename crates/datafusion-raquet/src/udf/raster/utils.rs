// use rastertile_rs::{BinaryType, CompressionFormat, DataType as RasterDataType, Operations};
// use rastertile_schema::Metadata;

// use crate::error::RaquetDataFusionResult;
// use arrow::array::GenericListBuilder;
// use arrow_array::builder::{
//     Float32Builder, Float64Builder, GenericBinaryBuilder, ListBuilder, PrimitiveBuilder,
//     UInt8Builder,
// };
// use arrow_array::{
//     ArrayRef, BinaryArray, ListArray,
//     types::{Float32Type, Float64Type},
// };

// // fn get_data_type_from_metadata(metadata: Metadata) -> Option<NewDataType> {
// //     let data_type: Option<NewDataType> = match metadata.data_type() {
// //         RasterDataType::UInt8 => Some(NewDataType::UInt8),
// //         RasterDataType::Int8 => Some(NewDataType::Int8),
// //         RasterDataType::UInt16 => Some(NewDataType::UInt16),
// //         RasterDataType::Int16 => Some(NewDataType::Int16),
// //         RasterDataType::UInt32 => Some(NewDataType::UInt32),
// //         RasterDataType::Int32 => Some(NewDataType::Int32),
// //         RasterDataType::UInt64 => Some(NewDataType::UInt64),
// //         RasterDataType::Int64 => Some(NewDataType::Int64),
// //         RasterDataType::Float32 => Some(NewDataType::Float32),
// //         RasterDataType::Float64 => Some(NewDataType::Float64),
// //     };
// //     data_type
// // }

// // pub fn get_tile(metadata: Metadata, data: Option<&[u8]>) -> Tile {
// //     let samples = match metadata.bands() {
// //         Some(bands) => bands.len(),
// //         _ => 1,
// //     };
// //     let tile: Tile = Tile {
// //         x: metadata.tile_size().clone(),
// //         y: metadata.tile_size().clone(),
// //         data_type: get_data_type_from_metadata(metadata.clone()),
// //         compressed_bytes: data.unwrap().to_vec(),
// //         compression_method: metadata.compression().clone(),
// //         samples,
// //     };
// //     tile
// //     // let psize = match metadata.clone().bands {
// //     //     Some(a) => a.len(),
// //     //     _ => 0,
// //     // };
// //     // match metadata.binary_type().clone() {
// //     //     BinaryType::Separated => {
// //     //         let tile: Tile = Tile {
// //     //             x: metadata.tile_size().clone(),
// //     //             y: metadata.tile_size().clone(),
// //     //             data_type: get_data_type_from_metadata(metadata.clone()),
// //     //             compressed_bytes: data.unwrap().to_vec(),
// //     //             compression_method: metadata.compression().clone(),
// //     //         };
// //     //         tile
// //     //     }
// //     //     BinaryType::Interleaved => {
// //     //         let tile: Tile = Tile {
// //     //             x: metadata.tile_size().clone() * psize,
// //     //             y: metadata.tile_size().clone() * psize,
// //     //             data_type: get_data_type_from_metadata(metadata.clone()),
// //     //             compressed_bytes: data.unwrap().to_vec(),
// //     //             compression_method: metadata.compression().clone(),
// //     //         };
// //     //         tile
// //     //     }
// //     // }
// // }

// //   let tile: Tile = Tile {
// //         x: metadata.tile_size().clone(),
// //         y: metadata.tile_size().clone(),
// //         data_type: get_data_type_from_metadata(metadata.clone()),
// //         compressed_bytes: data.unwrap().to_vec(),
// //         compression_method: metadata.compression().clone(),
// //     };
// //     let decoded_array = tile.decode().unwrap();
// //     crate::udf::raster::utils::convert_f32(decoded_array.data())

// // #[macro_export]
// // macro_rules! impl_convert_list_array {
// //     ($in_type:ident, $out_type:ident,$builder:ident, $name:ident) => {
// //         pub fn $name(
// //             in_binary: &arrow_array::GenericByteArray<arrow::datatypes::GenericBinaryType<i32>>,
// //             metadata: Metadata,
// //         ) -> RaquetDataFusionResult<ListArray> {
// //             let values_builder = $builder::new();
// //             let mut builder = ListBuilder::new(values_builder);

// //             for input in in_binary.iter() {
// //                 let tile: Tile = get_tile(metadata.clone(), input);
// //                 let decoded_array = tile.decode().unwrap();
// //                 let output: Vec<Option<$out_type>> = match decoded_array.data() {
// //                     TypedArray::$in_type(v) => {
// //                         let out = v.iter().map(|n| Some(n.clone())).collect();
// //                         out
// //                     }
// //                     _ => panic!("expected Float32"),
// //                 };

// //                 // }

// //                 builder.append_value(output);
// //             }

// //             let arr = builder.finish();

// //             Ok(arr)
// //         }
// //     };
// // }

// // #[macro_export]
// // macro_rules! impl_convert_array {
// //     ($in_type:ident, $out_type:ident, $name:ident) => {
// //         pub fn $name(data: &TypedArray) -> Vec<Option<$out_type>> {
// //             let vals = match data {
// //                 TypedArray::$in_type(v) => {
// //                     let out = v.iter().map(|n| Some(n.clone())).collect();
// //                     out
// //                 }
// //                 _ => panic!("expected Float32"),
// //             };
// //             vals
// //         }
// //     };
// // }

// // pub fn convert_list_array_f32(in_binary: BinaryArray, metadata: Metadata) -> ListArray {
// //     let values_builder = Float32Builder::new();
// //     let mut builder = ListBuilder::new(values_builder);

// //     let ops: Operations = Operations::new(metadata.inner());

// //     for input in in_binary.iter() {
// //         ops.
// //         let tile: Tile = get_tile(metadata.clone(), input);
// //         let decoded_array = tile.decode().unwrap();
// //         let output: Vec<Option<f32>> = convert_f32(decoded_array.data());

// //         builder.append_value(output);
// //     }

// //     let arr = builder.finish();

// //     arr
// // }

// // pub fn get_pixel(input: Option<&[u8]>, metadata: Metadata, x: u64, y: u64) -> f64 {
// //     let tile: Tile = get_tile(metadata.clone(), input);
// //     tile.get_pixel(x, y)
// // }

// pub fn decompress_tile(input: Option<&[u8]>, metadata: Metadata) -> Vec<u8> {
//      let ops: Operations = Operations::new(metadata.inner());
 
//     let decompressed = ops.decompress(input)
//     decompressed
// }

// // pub fn decompress(in_binary: BinaryArray, metadata: Metadata) {
// //     let mut builder = GenericBinaryBuilder::<i32>::new();

// //     for input in in_binary.iter() {
// //         let tile: Tile = get_tile(metadata.clone(), input);
// //         let decompressed = tile.decompress().unwrap();
// //         builder.append_value(decompressed);
// //     }

// //     let point_arr = builder.finish();
// // }

// // pub fn convert_list_array_u8(in_binary: BinaryArray, metadata: Metadata) -> ListArray {
// //     let values_builder = UInt8Builder::new();
// //     let mut builder = ListBuilder::new(values_builder);

// //     for input in in_binary.iter() {
// //         let tile: Tile = get_tile(metadata.clone(), input);
// //         let decoded_array = tile.decode().unwrap();
// //         let output: Vec<Option<u8>> = convert_uint8(decoded_array.data());

// //         builder.append_value(output);
// //     }

// //     let arr = builder.finish();

// //     arr
// // }

// // impl_convert_array!(Float32, f32, convert_f32);
// // impl_convert_array!(UInt8, u8, convert_uint8);
// // // impl_convert_list_array!(Float32, f32,Float32Builder, convert_list_array_f32);
// // impl_convert_list_array!(Float64, f64, Float64Builder, convert_list_array_f64);
// // // impl_table_provider!(RaquetTable, PyRaquetTable, "RaquetTable");
