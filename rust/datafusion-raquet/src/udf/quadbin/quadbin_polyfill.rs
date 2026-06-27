use std::sync::{Arc, OnceLock};

use arrow_array::builder::{ListBuilder, UInt64Builder};
use arrow_array::cast::{as_primitive_array, as_string_array};

use arrow_array::{ArrayRef, Int64Array, ListArray, UInt64Array};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

use crate::error::RaquetDataFusionResult;

use quadbin_geo_rs::GeoCells;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct QuadBinPolyFill {
    signature: Signature,
}

impl QuadBinPolyFill {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(
                vec![DataType::Utf8, DataType::Int64],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for QuadBinPolyFill {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for QuadBinPolyFill {
    fn name(&self) -> &str {
        "quadbin_polyfill"
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
        let cell_arr = build_cell_array(arrays)?;
        let array_ref: ArrayRef = Arc::new(cell_arr);
        Ok(ColumnarValue::Array(array_ref))
    }

    fn documentation(&self) -> Option<&Documentation> {
        Some(DOCUMENTATION.get_or_init(|| {
            Documentation::builder(
                DOC_SECTION_OTHER,
                "Return a List[QUADBIN] of cell children from a wkt geometry with a resolution",
                "quadbin_polyfill('POLYGON((-45 40.9798980696201, 0 40.9798980696201, 0 66.5132604431119, -45 66.5132604431119, -45 40.9798980696201))',4) or quadbin_polyfill(5256690695657226239,13)",
            )
            .with_argument("cell", "cell value")
            .with_argument("resolution", "resolution value")
            .build()
        }))
    }
}

fn return_field_impl(_args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    let list_field = Field::new_list_field(DataType::UInt64, true);
    let dt = DataType::List(Arc::new(list_field));
    let out_field: Field = Field::new("", dt, true);

    Ok(Arc::new(out_field))
}

fn build_cell_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<ListArray> {
    // as_string_array( arrays.get(0));
    let wkt_array = as_string_array(&arrays[0]);
    let resolution_array: &Int64Array = as_primitive_array(&arrays[1]);
    let values_builder = UInt64Builder::new();
    let mut builder = ListBuilder::new(values_builder);

    for (wkt, resolution) in wkt_array.iter().zip(resolution_array.iter()) {
        let geocells = GeoCells::new(wkt.unwrap().to_string(), resolution.unwrap() as i8)
            .intersecting_cells()?;
        let bounding = UInt64Array::from(geocells);

        builder.append_value(&bounding);
    }

    let point_arr = builder.finish();

    Ok(point_arr)
}

#[cfg(test)]
mod tests {
    use datafusion::prelude::SessionContext;

    use super::*;

    #[tokio::test]
    async fn test_quadbin_to_tile() {
        let ctx = SessionContext::new();
        ctx.register_udf(QuadBinPolyFill::default().into());
        let sql = r#"select quadbin_polyfill('POLYGON((-74.1 40.6, -73.8 40.6, -73.8 40.9, -74.1 40.9, -74.1 40.6))',12) ;"#;
        println!("{:?}", sql);

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }
}
