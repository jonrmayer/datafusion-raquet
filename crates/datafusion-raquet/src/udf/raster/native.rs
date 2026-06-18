use std::any::Any;
use std::sync::{Arc, OnceLock};

use crate::error::RaquetDataFusionResult;
use arrow::array::GenericListBuilder;
use arrow_array::builder::{Float32Builder, Float64Builder, ListBuilder, PrimitiveBuilder};
use arrow_array::{
    ArrayRef, BinaryArray, ListArray,
    types::{Float32Type, Float64Type},
};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

use datafusion_sql::sqlparser::ast::DataType::Float32;
use rasterarrow_schema::Metadata;

use rastertile_rs::{CompressionFormat, NewDataType, RasterDataType, Tile, TypedArray};

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

fn return_field_impl(args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    let metadata = Arc::new(Metadata::try_from(args.arg_fields[0].as_ref()).unwrap_or_default());

    let list_field = match metadata.data_type() {
        RasterDataType::UInt8 => Some(Field::new_list_field(DataType::UInt8, true)),
        // RasterDataType::Int8 => Some(NewDataType::Int8),
        // RasterDataType::UInt16 => Some(NewDataType::UInt16),
        // RasterDataType::Int16 => Some(NewDataType::Int16),
        // RasterDataType::UInt32 => Some(NewDataType::UInt32),
        // RasterDataType::Int32 => Some(NewDataType::Int32),
        // RasterDataType::UInt64 => Some(NewDataType::UInt64),
        // RasterDataType::Int64 => Some(NewDataType::Int64),
        RasterDataType::Float32 => Some(Field::new_list_field(DataType::Float32, true)),
        RasterDataType::Float64 => Some(Field::new_list_field(DataType::Float64, true)),
        _ => None,
    };

    let dt = DataType::List(Arc::new(list_field.unwrap()));
    let out_field: Field = Field::new("", dt, true);

    Ok(Arc::new(out_field))
}

fn build_cell_array(
    arrays: Vec<ArrayRef>,
    metadata: Metadata,
) -> RaquetDataFusionResult<ListArray> {
    let in_binary = arrays[0]
        .as_any()
        .downcast_ref::<BinaryArray>()
        .expect("cast failed");

    let out = match metadata.data_type() {
        RasterDataType::UInt8 => Some(crate::udf::raster::convert_list_array_u8(
            in_binary.clone(),
            metadata,
        )),
        // RasterDataType::Int8 => Some(NewDataType::Int8),
        // RasterDataType::UInt16 => Some(NewDataType::UInt16),
        // RasterDataType::Int16 => Some(NewDataType::Int16),
        // RasterDataType::UInt32 => Some(NewDataType::UInt32),
        // RasterDataType::Int32 => Some(NewDataType::Int32),
        // RasterDataType::UInt64 => Some(NewDataType::UInt64),
        // RasterDataType::Int64 => Some(NewDataType::Int64),
        RasterDataType::Float32 => Some(crate::udf::raster::convert_list_array_f32(
            in_binary.clone(),
            metadata,
        )),
        // RasterDataType::Float64 => Some(crate::udf::raster::convert_list_array_f64(
        //     in_binary, metadata,
        // )),
        _ => None,
    };
    Ok(out.unwrap())
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::RaquetTable;
    use crate::udf::raster::DecodeTile;
    use datafusion::prelude::{SessionConfig, SessionContext};

    #[tokio::test]
    async fn test_native_interleaved_tile() {
        let path =
            "/home/jonrm/projects/git/raquet-datafusion/data/parquet/tci_interleaved_gzip.parquet"
                .to_string();

        let ctx =
            SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

        ctx.register_udf(NativeTile::default().into());
        ctx.register_udf(DecodeTile::default().into());
        let t = RaquetTable::from_path(path).await;

        let _ = ctx.register_table("tci", Arc::new(t));
        // let tci = ctx.table_provider("tci").await.unwrap();

        // let tci_schema = tci.schema();
        // println!("{:?}", tci_schema);

        let sql = "select array_length(native_tile(pixels),1) from tci where block<>0 limit 1 ;";
        // let sql = "select count(*) from solar;";

        let df = ctx.sql(sql).await.unwrap();
        // println!("{:?}",df.count().await);
        df.show().await.unwrap();
    }
}
