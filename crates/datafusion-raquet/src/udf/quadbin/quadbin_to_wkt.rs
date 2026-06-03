use std::any::Any;
use std::sync::{Arc, OnceLock};

use arrow_array::builder::StringViewBuilder;
use arrow_array::cast::AsArray;
use arrow_array::types::{Int64Type, UInt8Type, UInt32Type, UInt64Type};
use arrow_array::{Array, ArrayRef, GenericListArray, ListArray, StructArray, UInt64Array};
use arrow_schema::{DataType, Field, FieldRef, Fields};

use arrow_convert::{
    ArrowDeserialize, ArrowField, ArrowSerialize, deserialize::TryIntoCollection,
    serialize::TryIntoArrow,
};

use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    TypeSignature, Volatility,
};

use crate::error::{RaquetDataFusionError, RaquetDataFusionResult};

use crate::udf::quadbin::converter::{Abbox, LonLat, Pixel};
use quadbin_rs::{Tile, cell_to_tile, tile_to_bbox_wgs84};

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct QuadBinToWKT {
    signature: Signature,
}

impl QuadBinToWKT {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(vec![DataType::Int64], Volatility::Immutable),
        }
    }
}

impl Default for QuadBinToWKT {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for QuadBinToWKT {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "quadbin_to_wkt"
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
        let arrays = ColumnarValue::values_to_arrays(&args.args)?;
        let cell_arr = build_wkt_array(arrays)?;
        // let array_ref: ArrayRef = Arc::new(cell_arr);
        Ok(cell_arr)
    }

    fn documentation(&self) -> Option<&Documentation> {
        Some(DOCUMENTATION.get_or_init(|| {
            Documentation::builder(
                DOC_SECTION_OTHER,
                "Return a WKT String from a quadbin cell ",
                "quadbin_to_wkt(5256690695657226239) ",
            )
            .with_argument("cell", "cell value")
            .build()
        }))
    }
}

fn return_field_impl(_args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    Ok(Arc::new(Field::new("", DataType::Utf8View, false)))
}

fn build_wkt_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<ColumnarValue> {
    let cells = arrays[0].as_primitive::<Int64Type>();
    let mut builder = StringViewBuilder::with_capacity(cells.len());

    for cell in cells.iter() {
        let tile: Tile = cell_to_tile(cell.unwrap() as u64);
        let bbox = tile_to_bbox_wgs84(tile);
        let wkt = bbox_to_wkt(bbox);
        builder.append_value(wkt);
    }

    Ok(ColumnarValue::Array(Arc::new(builder.finish())))
}

#[cfg(test)]
mod tests {
    use datafusion::prelude::SessionContext;

    use super::*;

    #[tokio::test]
    async fn test_quadbin_to_children() {
        let ctx = SessionContext::new();
        ctx.register_udf(QuadBinToWKT::default().into());
        let sql = r#"SELECT quadbin_to_bbox_mercator(5256690695657226239) ;"#;
        println!("{:?}", sql);

        let df = ctx.sql(sql).await.unwrap();
        // df.show();
        let batches = df.collect().await.unwrap();
        // let column = batches[0].column(0);
        // // let string_arr = column.as_string_view();

        // let val = column.as_list(0).value(0);
        // println!("{:?}", val);
    }

    #[tokio::test]
    async fn test_quadbin_to_parent_resolution() {
        let ctx = SessionContext::new();
        ctx.register_udf(QuadBinToWKT::default().into());
        let sql = r#"SELECT quadbin_to_children(5256690695657226239,13) cell;"#;
        println!("{:?}", sql);

        let df = ctx.sql(sql).await.unwrap();
        let batches = df.collect().await.unwrap();
        let column = batches[0].column(0);
        // let string_arr = column.as_string_view();

        let val = column.as_primitive::<UInt64Type>().value(0);
        println!("{:?}", val);
    }
}
