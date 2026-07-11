use std::sync::{Arc, OnceLock};
use std::any::Any;

use crate::error::RaquetDataFusionResult;
use arrow_array::builder::GenericBinaryBuilder;
use arrow_array::cast::as_string_array;
use arrow_array::{ArrayRef, BinaryArray, BinaryViewArray, StringViewArray};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    TypeSignature, Volatility,
};

use rastertile_schema::{Metadata, RasterType,RasterArrowType};

use rastertile_rs::{CompressionFormat, Operations};

use crate::udf::raster::utils::has_extension;
use crate::{raquet_band_metadata, raquet_format_from_str};

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct DecompressTile {
    signature: Signature,
}

impl DecompressTile {
    pub fn new() -> Self {
        Self {
            signature: Signature::one_of(
                vec![
                    TypeSignature::Exact(vec![DataType::Binary]),
                    TypeSignature::Exact(vec![DataType::Binary, DataType::Utf8]),
                    TypeSignature::Exact(vec![DataType::BinaryView, DataType::Utf8View]),
                ],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for DecompressTile {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for DecompressTile {
         fn as_any(&self) -> &dyn Any {
        self
    }
    fn name(&self) -> &str {
        "raquet_band_decompress"
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
        let binary_field = &args.arg_fields[0];
        let binary_type = binary_field.data_type();
        let binary_name = binary_field.name();
        let arrays = ColumnarValue::values_to_arrays(&args.args)?;
        let cell_arr = match has_extension(args.arg_fields[0].as_ref()) {
            true => {
                let existing_metadata =
                    Metadata::try_from(args.arg_fields[0].as_ref()).unwrap_or_default();
                build_cell_array(arrays, binary_name, binary_type, Some(existing_metadata))?
            }
            false => build_cell_array(arrays, binary_name, binary_type, None)?,
        };

        let array_ref: ArrayRef = Arc::new(cell_arr);
        Ok(ColumnarValue::Array(array_ref))
    }

    fn documentation(&self) -> Option<&Documentation> {
        Some(DOCUMENTATION.get_or_init(|| {
            Documentation::builder(
                DOC_SECTION_OTHER,
                "Return a decompressed binary from an compressed binary.",
                "raquet_band_decompress(band_1,metadata)",
            )
            .with_argument("band", "band value")
            .with_argument("metadata", "optional metadata value")
            .build()
        }))
    }
}

fn return_field_impl(args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    let binary_field = &args.arg_fields[0];
    match has_extension(args.arg_fields[0].as_ref()) {
        true => {
            let existing_metadata = Metadata::try_from(binary_field.as_ref()).unwrap_or_default();
            let new_metadata = Metadata::new(
                existing_metadata.tile_size(),
                existing_metadata.binary_type(),
                existing_metadata.data_type(),
                existing_metadata.no_data(),
                CompressionFormat::None,
                existing_metadata.bands(),
            );
            let metadata = Arc::new(new_metadata);
            let raster_type = RasterType::new(metadata);
            let raster_arrow_type = match binary_field.data_type() {
                DataType::Binary => RasterArrowType::Raster(raster_type),
                DataType::LargeBinary => RasterArrowType::LargeRaster(raster_type),
                DataType::BinaryView => RasterArrowType::RasterView(raster_type),
                _ => unreachable!(),
            };
            Ok(Arc::new(raster_arrow_type.to_field("", true)))
        }
        false => Ok(Arc::new(Field::new("", DataType::Binary, false))),
    }
}

fn build_cell_array(
    arrays: Vec<ArrayRef>,
    binary_name: &String,
    binary_type: &DataType,
    metadata: Option<Metadata>,
) -> RaquetDataFusionResult<BinaryArray> {
    let mut builder = GenericBinaryBuilder::<i32>::new();

    match (binary_type, metadata) {
        (DataType::Binary, Some(m)) => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryArray>()
                .expect("cast failed");
            let ops: Operations = Operations::new(m.inner());
            for input in in_binary.iter() {
                let output = ops.decompress(input)?;

                builder.append_value(output);
            }
        }
        (DataType::Binary, None) => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryArray>()
                .expect("cast failed");
            let metadata_array = as_string_array(&arrays[1]);
            for (input, metadata) in in_binary.iter().zip(metadata_array.iter()) {
                let rcm = raquet_band_metadata(binary_name, metadata.unwrap());
                let ops: Operations = Operations::new(rcm);
                let output = ops.decompress(input)?;

                builder.append_value(output);
            }
        }
        (DataType::BinaryView, Some(m)) => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryViewArray>()
                .expect("cast failed");
            let ops: Operations = Operations::new(m.inner());
            for input in in_binary.iter() {
                let output = ops.decompress(input)?;

                builder.append_value(output);
            }
        }
        (DataType::BinaryView, None) => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryViewArray>()
                .expect("cast failed");
            let metadata_array = arrays[1]
                .as_any()
                .downcast_ref::<StringViewArray>()
                .expect("cast failed");
            for (input, metadata) in in_binary.iter().zip(metadata_array.iter()) {
                let rcm = raquet_band_metadata("band_1", metadata.unwrap());
                let ops: Operations = Operations::new(rcm);
                let output = ops.decompress(input)?;

                builder.append_value(output);
            }
        }
        _ => unreachable!(),
    }

    let point_arr = builder.finish();

    Ok(point_arr)
}

// #[cfg(test)]
// mod tests {
//     use crate::RaquetTable;
//     use crate::register;
//     use crate::views::ReadRaquetMetadata;
//     use datafusion::prelude::*;
//     use datafusion::prelude::{SessionConfig, SessionContext};

//     use super::*;
//     pub async fn setup_local() -> SessionContext {
//         let path =
//         "file:///home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet"
//             .to_string();

//         let mut ctx =
//             SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

//         // register(&mut ctx);
//         ctx.register_udf(DecompressTile::default().into());

//         let t = RaquetTable::from_path(path).await;

//         let _ = ctx.register_table("solar", Arc::new(t));
//         ctx
//     }

//     pub async fn setup_local_parquet() -> SessionContext {
//         let path =
//             "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet";

//         let ctx = SessionContext::new();
//         // SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

//         // register(&mut ctx);
//         ctx.register_udf(DecompressTile::default().into());
//         let _ = ctx
//             .register_parquet("solar", path, ParquetReadOptions::default())
//             .await
//             .map_err(|e| {
//                 DataFusionError::Context(format!("Registering 'hits_raw' as {path}"), Box::new(e))
//             });

//         ctx
//     }

//     #[tokio::test]
//     async fn test_decompress_tile() {
//         let path =
//             "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet"
//                 .to_string();

//         let ctx =
//             SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

//         ctx.register_udf(DecompressTile::default().into());
//         let t = RaquetTable::from_path(path).await;

//         let _ = ctx.register_table("solar", Arc::new(t));

//         let sql = r#"SELECT raquet_band_decompress(band_1) from solar where block<>0  limit 1 ;"#;
//         println!("{:?}", sql);

//         let df = ctx.sql(sql).await.unwrap();
//         println!("{:?}", df.count().await);
//         // df.show().await.unwrap();
//     }

//     #[tokio::test]
//     async fn test_decompress_tile2() {
//         let ctx = setup_local().await;
//         let sql = r###"
//         with m as (
//             select metadata from solar where block=0

//         ),
//         data as (
//             select band_1  from solar
//             where block<>0 limit 1
//         )

//         select raquet_band_decompress(data.band_1,m.metadata) from data,m
       
       

   
//     "###;

//         let df = ctx.sql(sql).await.unwrap();
//         df.show().await.unwrap();
//     }

//     #[tokio::test]
//     async fn test_decompress_tile_parquet() {
//         let ctx = setup_local_parquet().await;
//         let sql = r###"
//         with m as (
//             select metadata from solar where block=0

//         ),
//         data as (
//             select band_1  from solar
//             where block<>0 limit 1
//         )

//         select raquet_band_decompress(data.band_1,m.metadata) from data,m
       
       

   
//     "###;

//         let df = ctx.sql(sql).await.unwrap();
//         df.show().await.unwrap();
//     }
// }
