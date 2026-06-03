use std::any::Any;
use std::sync::{Arc, OnceLock};

use crate::error::RaquetDataFusionResult;
use arrow_array::builder::{Float32Builder, ListBuilder};
use arrow_array::{ArrayRef, BinaryArray, ListArray};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

use rasterarrow_schema::Metadata;

use rastertile_rs::{CompressionFormat, NewDataType, Tile,  TypedArray};

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct NativeTile {
    signature: Signature,
}

impl NativeTile {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(vec![DataType::Binary], Volatility::Immutable),
        }
    }
}

impl Default for NativeTile {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for NativeTile {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "native_tile"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        Err(DataFusionError::Internal("return_type".to_string()))
    }

    fn return_field_from_args(&self, args: ReturnFieldArgs) -> Result<FieldRef> {
        Ok(return_field_impl(args)?)
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        // let field = &args.arg_fields[0];
        // let to_type = RasterArrowType::from_arrow_field(args.return_field.as_ref()).unwrap();
        let existing_metadata = Metadata::try_from(args.arg_fields[0].as_ref()).unwrap_or_default();
        let arrays = ColumnarValue::values_to_arrays(&args.args)?;
        let cell_arr = build_cell_array(arrays, existing_metadata)?;
        let array_ref: ArrayRef = Arc::new(cell_arr);
        Ok(ColumnarValue::Array(array_ref))
    }

    fn documentation(&self) -> Option<&Documentation> {
        Some(DOCUMENTATION.get_or_init(|| {
            Documentation::builder(
                DOC_SECTION_OTHER,
                "Return a decoded binary from an encoded binary.",
                "decode_tile(tile)",
            )
            .with_argument("tile", "tile value")
            .build()
        }))
    }
}

fn return_field_impl(_args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    let lf = Field::new_list_field(DataType::Float32, true);
    let dt = DataType::List(Arc::new(lf));
    let out_field: Field = Field::new("", dt, true);
    // let input_field = &args.arg_fields[0];
    // let existing_metadata = Arc::new(Metadata::try_from(input_field.as_ref())?);
    // let new_metadata = Metadata::new(
    //     existing_metadata.tile_size,
    //     existing_metadata.binary_type,
    //     existing_metadata.data_type,
    //     CompressionFormat::None,
    // );
    // let metadata = Arc::new(new_metadata);
    // let values_field: Field = Field::new("", DataType::Float32, true);
    // let dt = DataType::List(Arc::new(values_field));
    // Field::
    // let out_field: Field = Field::
    // let output_type = RasterFloat32Type::new(metadata);
    Ok(Arc::new(out_field))
    // Ok(raster_type
    //     .to_field(input_field.name(), input_field.is_nullable())
    //     .into())
}
fn convert(_metadata: Metadata, data: Option<&[u8]>) -> Vec<Option<f32>> {
    let tile: Tile = Tile {
        x: 256,
        y: 256,
        data_type: Some(NewDataType::Float32),
        compressed_bytes: data.unwrap().to_vec(),
        compression_method: CompressionFormat::Gzip,
    };
    let a = tile.decode().unwrap();
    let bb = a.data().clone();

    let vals: Vec<Option<f32>> = match bb {
        TypedArray::Float32(v) => {
            let out = v.iter().map(|n| Some(n.clone())).collect();
            out
        }
        _ => panic!("expected Float32"),
    };
   
    vals
}

fn build_cell_array(
    arrays: Vec<ArrayRef>,
    metadata: Metadata,
) -> RaquetDataFusionResult<ListArray> {
    let in_binary = arrays[0]
        .as_any()
        .downcast_ref::<BinaryArray>()
        .expect("cast failed");

    let values_builder = Float32Builder::new();
    let mut builder = ListBuilder::new(values_builder);

    for input in in_binary.iter() {
        let output = convert(metadata.clone(), input);
        builder.append_value(output);
    }

    let point_arr = builder.finish();

    Ok(point_arr)
}

// #[cfg(test)]
// mod tests {
//     use arrow_array::RecordBatch;
//     use arrow_buffer::ScalarBuffer;
//     use arrow_schema::Schema;
//     use datafusion::prelude::SessionContext;

//     use std::fs::File;
//     use std::path::Path;
//     use std::path::PathBuf;
//     use std::sync::Arc;

//     use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
//     use parquet::file::reader::{FileReader, SerializedFileReader};

//     use raquet::reader::{RaquetFileReader, RaquetReaderBuilder, RaquetRecordBatchReader};
//     // use parquet::record::{Field, Row, RowAccessor};

//     use super::*;
//     pub(crate) fn fixture_dir() -> PathBuf {
//         let p = PathBuf::from("/home/jonrm/projects/git/raquet-arrow/fixtures");
//         assert!(p.exists());
//         p
//     }

//     pub(crate) fn spain_solar_ghi() -> PathBuf {
//         fixture_dir().join("spain_solar_ghi.parquet")
//     }

//     // pub(crate) fn geoarrow_data_example_crs_files() -> PathBuf {
//     //     fixture_dir().join("geoarrow-data/example-crs/files")
//     // }

//     fn read_gpq_file(path: impl AsRef<Path>) -> Vec<RecordBatch> {
//         println!("reading path: {:?}", path.as_ref());
//         // path.
//         let inner_file = File::open(spain_solar_ghi()).unwrap();
//         let outer_file = File::open(spain_solar_ghi()).unwrap();

//         let reader = SerializedFileReader::new(inner_file).unwrap();
//         let fm = reader.metadata().file_metadata();
//         // reader.

//         let raquet_file_metadata = reader.raquet_file_metadata().unwrap().unwrap();
//         let format = reader.raquet_format().unwrap().unwrap();

//         let reader_builder = ParquetRecordBatchReaderBuilder::try_new(outer_file).unwrap();
//         let raquet_meta = reader_builder.raquet_metadata(format).unwrap().unwrap();
//         let raquet_schema = reader_builder.raquet_schema(&raquet_meta, true).unwrap();
//         // reader_builder.with_row_groups(raquet_file_metadata.raquet_row_groups)
//         let pq_reader = reader_builder
//             .with_row_groups(raquet_file_metadata.raquet_row_groups())
//             .build()
//             .unwrap();

//         let reader = RaquetRecordBatchReader::try_new(pq_reader, raquet_schema.clone()).unwrap();
//         let batches = reader.collect::<Result<Vec<_>, _>>().unwrap();

//         batches
//     }

//     //   #[tokio::test]
//     // async fn test_list_array() {
//     //     let list_array = get_la().unwrap();
//     //     let v = list_array.value(0);

//     //     // let s = ScalarBuffer::from(v.to_data());
//     //     // println!("{:?}", v.to_data().into_parts());
//     //     // list_array.values()
//     //     // let path = spain_solar_ghi();
//     //     // let batch = read_gpq_file(path);
//     //     // let ctx = SessionContext::new();
//     //     // ctx.read_batches(batches)
//     //     // ctx.re
//     //     // ctx.register_batch("solar", batch[0].clone());
//     //     // ctx.register_udf(NativeTile::default().into());

//     //     // let sql = r#"SELECT native_tile(band_1) decoded_tile from solar ;"#;
//     //     // // println!("{:?}", sql);

//     //     // let df = ctx.sql(sql).await.unwrap();
//     //     // // let schema = df.schema();

//     //     // let schema = df.schema().clone();
//     //     // // schema.field(0).metadata();
//     //     // println!("{:?}", schema.field(0).metadata());
//     //     // let batches = df.collect().await.unwrap();
//     //     // let column = batches[0].column(0);
//     //     // // column.type_id();

//     //     // let val = column.as_primitive::<BinaryType>().value(0);
//     // }

//     #[tokio::test]
//     async fn test_native_tile() {
//         let path = spain_solar_ghi();
//         let batch = read_gpq_file(path);
//         let ctx = SessionContext::new();
//         // ctx.read_batches(batches)
//         // ctx.re
//         ctx.register_batch("solar", batch[0].clone());
//         ctx.register_udf(NativeTile::default().into());

//         let sql = r#"SELECT native_tile(band_1) decoded_tile from solar limit 1 ;"#;
//         // println!("{:?}", sql);

//         let df = ctx.sql(sql).await.unwrap();
//         // let schema = df.schema();

//         let schema = df.schema().clone();
//         // schema.field(0).metadata();
//         println!("{:?}", schema.field(0).metadata());
//         let batches = df.collect().await.unwrap();
//         let column = batches[0].column(0);
//         // column.type_id();

//         // let val = column.as_primitive::<BinaryType>().value(0);
//     }
// }
