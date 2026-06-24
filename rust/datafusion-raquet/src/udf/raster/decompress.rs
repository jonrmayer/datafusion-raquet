use std::any::Any;
use std::sync::{Arc, OnceLock};

use crate::error::RaquetDataFusionResult;
use arrow_array::builder::GenericBinaryBuilder;
use arrow_array::{ArrayRef, BinaryArray};
use arrow_schema::{DataType, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

use rastertile_schema::{Metadata, RasterType};

use rastertile_rs::{Compression, CompressionFormat, Operations};

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct DecompressTile {
    signature: Signature,
}

impl DecompressTile {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(vec![DataType::Binary], Volatility::Immutable),
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
        "decompress_tile"
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
                "decompress_tile(tile)",
            )
            .with_argument("tile", "tile value")
            .build()
        }))
    }
}

fn return_field_impl(args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    let existing_metadata = Metadata::try_from(args.arg_fields[0].as_ref()).unwrap_or_default();
    let new_metadata = Metadata::new(
        existing_metadata.tile_size(),
        existing_metadata.binary_type(),
        existing_metadata.data_type(),
        existing_metadata.no_data(),
        CompressionFormat::None,
        existing_metadata.bands(),
    );
    let metadata = Arc::new(new_metadata);
    let output_type = RasterType::new(metadata);
    Ok(Arc::new(output_type.to_field("", true)))
}


fn build_cell_array(
    arrays: Vec<ArrayRef>,
    metadata: Metadata,
) -> RaquetDataFusionResult<BinaryArray> {
    let in_binary = arrays[0]
        .as_any()
        .downcast_ref::<BinaryArray>()
        .expect("cast failed");

    let mut builder = GenericBinaryBuilder::<i32>::new();

    let ops: Operations = Operations::new(metadata.inner());

    for input in in_binary.iter() {
        let output = ops.decompress(input)?;

        builder.append_value(output);
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
    async fn test_decompress_tile() {
        let path =
            "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet"
                .to_string();

        let ctx =
            SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

        ctx.register_udf(DecompressTile::default().into());
        let t = RaquetTable::from_path(path).await;

        let _ = ctx.register_table("solar", Arc::new(t));
      

        let sql = r#"SELECT decompress_tile(band_1) from solar where block<>0  ;"#;
        println!("{:?}", sql);

        let df = ctx.sql(sql).await.unwrap();
        println!("{:?}",df.count().await);
        // df.show().await.unwrap();
    }
}
