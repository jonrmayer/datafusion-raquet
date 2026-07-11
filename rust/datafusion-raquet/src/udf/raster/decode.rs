use std::sync::{Arc, OnceLock};
use std::any::Any;

use crate::error::RaquetDataFusionResult;
use arrow_array::builder::{Float64Builder, ListBuilder};
use arrow_array::cast::as_string_array;
use arrow_array::{ArrayRef, BinaryArray, BinaryViewArray, ListArray, StringViewArray};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    TypeSignature, Volatility,
};

use rastertile_schema::Metadata;

use rastertile_rs::Operations;

use crate::udf::raster::utils::has_extension;
use crate::{raquet_band_metadata, raquet_format_from_str};

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct DecodeTile {
    signature: Signature,
}

impl DecodeTile {
    pub fn new() -> Self {
        Self {
            // signature: Signature::exact(vec![DataType::Binary], Volatility::Immutable),
            signature: Signature::one_of(
                vec![
                    TypeSignature::Exact(vec![DataType::Binary]),
                    TypeSignature::Exact(vec![DataType::BinaryView]),
                    TypeSignature::Exact(vec![DataType::LargeBinary]),
                ],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for DecodeTile {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for DecodeTile {
         fn as_any(&self) -> &dyn Any {
        self
    }
    fn name(&self) -> &str {
        "decode_tile"
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

        let cell_arr = match has_extension(binary_field.as_ref()) {
            true => {
                let existing_metadata =
                    Metadata::try_from(binary_field.as_ref()).unwrap_or_default();
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
                "Return a decoded binary from an encoded binary.",
                "decode_tile(tile)",
            )
            .with_argument("tile", "tile value")
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
    arrays: Vec<ArrayRef>,
    binary_name: &String,
    binary_type: &DataType,
    metadata: Option<Metadata>,
) -> RaquetDataFusionResult<ListArray> {
    let values_builder = Float64Builder::new();
    let mut builder = ListBuilder::new(values_builder);
    match (binary_type, metadata) {
        (DataType::Binary, Some(m)) => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryArray>()
                .expect("cast failed");
            let ops: Operations = Operations::new(m.inner());
            for input in in_binary.iter() {
                let output = ops.decode(input)?;

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
                let output = ops.decode(input)?;

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
                let output = ops.decode(input)?;

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
                let rcm = raquet_band_metadata(binary_name, metadata.unwrap());
                let ops: Operations = Operations::new(rcm);
                let output = ops.decode(input)?;

                builder.append_value(output);
            }
        }
        _ => unreachable!(),
    }

    let point_arr = builder.finish();

    Ok(point_arr)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::RaquetTable;
    use datafusion::prelude::{SessionConfig, SessionContext};

    #[tokio::test]
    async fn test_native_interleaved_tile() {
        let path =
            "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet"
                .to_string();

        let ctx =
            SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

        ctx.register_udf(DecodeTile::default().into());

        let t = RaquetTable::from_path(path).await;

        let _ = ctx.register_table("solar", Arc::new(t));

        let sql = "select decode_tile(band_1) from solar where block<>0   ;";
        let df = ctx.sql(sql).await.unwrap();
        println!("{:?}", df.count().await);
    }
}
