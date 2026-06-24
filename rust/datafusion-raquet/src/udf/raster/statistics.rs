use std::any::Any;
use std::sync::{Arc, OnceLock};

use crate::error::RaquetDataFusionResult;
use arrow::datatypes::Fields;
use arrow_array::builder::{Float64Builder, UInt64Builder};
use arrow_array::types::UInt64Type;
use arrow_array::{ArrayRef, BinaryArray, PrimitiveArray, StructArray};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};


use rastertile_schema::Metadata;

use rastertile_rs::Operations;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct StatisticsTile {
    signature: Signature,
}

impl StatisticsTile {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(vec![DataType::Binary], Volatility::Immutable),
        }
    }

    fn data_type(&self) -> DataType {
        let values_fields = vec![
            Field::new("min", DataType::Float64, false),
            Field::new("max", DataType::Float64, false),
            Field::new("mean", DataType::Float64, false),
            Field::new("std_dev", DataType::Float64, false),
            Field::new("valid_count", DataType::UInt64, false),
        ];
        DataType::Struct(values_fields.into())
    }
    fn to_field<N: Into<String>>(&self, name: N, nullable: bool) -> Field {
        Field::new(name, self.data_type(), nullable)
    }
}

impl Default for StatisticsTile {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for StatisticsTile {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "statistics_tile"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        Err(DataFusionError::Internal("return_type".to_string()))
    }

    fn return_field_from_args(&self, _args: ReturnFieldArgs) -> Result<FieldRef> {
        Ok(Arc::new(self.to_field("", false)))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
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

fn build_cell_array(
    arrays: Vec<ArrayRef>,
    metadata: Metadata,
) -> RaquetDataFusionResult<StructArray> {
    let in_binary = arrays[0]
        .as_any()
        .downcast_ref::<BinaryArray>()
        .expect("cast failed");

    let mut min_builder = Float64Builder::new();
    let mut max_builder = Float64Builder::new();
    let mut mean_builder = Float64Builder::new();
    let mut std_dev_builder = Float64Builder::new();
    let mut valid_count_builder = UInt64Builder::new();
    let ops: Operations = Operations::new(metadata.inner());
    for input in in_binary.iter() {
        let output = ops.statistics(input)?;
        min_builder.append_value(output.min);
        max_builder.append_value(output.max);
        mean_builder.append_value(output.mean);
        std_dev_builder.append_value(output.std_dev);
        valid_count_builder.append_value(output.valid_count);
    }

    let values_fields = vec![
        Field::new("min", DataType::Float64, false),
        Field::new("max", DataType::Float64, false),
        Field::new("mean", DataType::Float64, false),
        Field::new("std_dev", DataType::Float64, false),
        Field::new("valid_count", DataType::UInt64, false),
    ];

    let fields = Fields::from(values_fields);

    let arrays: Vec<ArrayRef> = vec![
        Arc::new(min_builder.finish()),
        Arc::new(max_builder.finish()),
        Arc::new(mean_builder.finish()),
        Arc::new(std_dev_builder.finish()),
        Arc::new(valid_count_builder.finish()),
    ];
    let nulls = None;
    let arr = StructArray::new(fields, arrays, nulls);
    Ok(arr)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::RaquetTable;
    use datafusion::prelude::{SessionConfig, SessionContext};

    #[tokio::test]
    async fn test_statistics_tile() {
        let path =
            "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet"
                .to_string();

        let ctx =
            SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

        ctx.register_udf(StatisticsTile::default().into());
        let t = RaquetTable::from_path(path).await;
        let _ = ctx.register_table("solar", Arc::new(t));

        let sql = "select statistics_tile(band_1) from solar where block = 5230520127799164927  ;";
      

        let df = ctx.sql(sql).await.unwrap();
        // println!("{:?}", df.count().await);
        df.show().await.unwrap();
    }
}
