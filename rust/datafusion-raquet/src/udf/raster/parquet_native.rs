use std::sync::{Arc, OnceLock};

use crate::error::RaquetDataFusionResult;
use arrow_array::builder::{Float64Builder, ListBuilder};
use arrow_array::{ArrayRef, BinaryArray, BinaryViewArray,StringViewArray,ListArray};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

use rastertile_schema::Metadata;

use crate::{raquet_band_metadata, raquet_format_from_str};
use rastertile_rs::Operations;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ParquetNativeTile {
    signature: Signature,
}

impl ParquetNativeTile {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(
                vec![DataType::BinaryView, DataType::Utf8View],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for ParquetNativeTile {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for ParquetNativeTile {
    fn name(&self) -> &str {
        "parquet_native_tile"
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

        let binary_name = binary_field.name();
        let arrays = ColumnarValue::values_to_arrays(&args.args)?;
        let cell_arr = build_cell_array(binary_name, arrays)?;
        let array_ref: ArrayRef = Arc::new(cell_arr);
        Ok(ColumnarValue::Array(array_ref))
    }

    fn documentation(&self) -> Option<&Documentation> {
        Some(DOCUMENTATION.get_or_init(|| {
            Documentation::builder(
                DOC_SECTION_OTHER,
                "Return a decoded binary from an encoded binary.",
                "parquet_decode_tile(band,metadata)",
            )
            .with_argument("band", "band value")
            .with_argument("metadata", "metadata value")
            .build()
        }))
    }
}

fn return_field_impl(_args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    let list_field = Field::new_list_field(DataType::Float64, true);
    let dt = DataType::List(Arc::new(list_field));
    let out_field: Field = Field::new("", dt, true);

    Ok(Arc::new(out_field))
}

fn build_cell_array(
    binary_name: &String,
    arrays: Vec<ArrayRef>,
) -> RaquetDataFusionResult<ListArray> {
    let in_binary = arrays[0]
        .as_any()
        .downcast_ref::<BinaryViewArray>()
        .expect("cast failed");
    let metadata_array = arrays[1]
        .as_any()
        .downcast_ref::<StringViewArray>()
        .expect("cast failed");
    let values_builder = Float64Builder::new();
    let mut builder = ListBuilder::new(values_builder);
    for (input, metadata) in in_binary.iter().zip(metadata_array.iter()) {
        let rcm = raquet_band_metadata(binary_name, metadata.unwrap());
        let ops: Operations = Operations::new(rcm);
        let output = ops.decode(input)?;

        builder.append_value(output);
    }

    let point_arr = builder.finish();

    Ok(point_arr)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::RaquetTable;
    use datafusion::prelude::*;
    use datafusion::prelude::{SessionConfig, SessionContext};

    pub async fn setup_local_parquet() -> SessionContext {
        let path =
            "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet";

        let ctx = SessionContext::new();
        // SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

        // register(&mut ctx);
        ctx.register_udf(ParquetDecodeTile::default().into());
        let _ = ctx
            .register_parquet("solar", path, ParquetReadOptions::default())
            .await
            .map_err(|e| {
                DataFusionError::Context(format!("Registering 'hits_raw' as {path}"), Box::new(e))
            });

        ctx
    }

    #[tokio::test]
    async fn test_decode_tile_parquet() {
        let ctx = setup_local_parquet().await;
        let sql = r###"
        with m as (
            select metadata from solar where block=0

        ),
        data as (
            select band_1  from solar
            where block<>0 limit 100
        )

        select parquet_decode_tile(data.band_1,m.metadata) from data,m
       
       

   
    "###;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_native_interleaved_tile() {
        let path =
            "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet"
                .to_string();

        let ctx =
            SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

        ctx.register_udf(ParquetDecodeTile::default().into());

        let t = RaquetTable::from_path(path).await;

        let _ = ctx.register_table("solar", Arc::new(t));

        let sql = "select decode_tile(band_1) from solar where block<>0   ;";
        let df = ctx.sql(sql).await.unwrap();
        println!("{:?}", df.count().await);
    }
}
