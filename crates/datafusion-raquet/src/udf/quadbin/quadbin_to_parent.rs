use std::any::Any;
use std::sync::{Arc, OnceLock};

use arrow_array::builder::UInt64Builder;
use arrow_array::cast::AsArray;
use arrow_array::types::Int64Type;
use arrow_array::{ArrayRef, UInt64Array};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    TypeSignature, Volatility,
};
// use geoarrow_array::GeoArrowArray;
// use geoarrow_array::array::GeometryArray;
// use geoarrow_array::builder::GeometryBuilder;
// use geoarrow_schema::{GeometryType, Metadata};

use crate::error::RaquetDataFusionResult;

use quadbin_rs::{cell_to_parent, cell_to_parent_resolution};

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct QuadBinToParent {
    signature: Signature,
}

impl QuadBinToParent {
    pub fn new() -> Self {
        Self {
            signature: Signature::one_of(
                vec![
                    TypeSignature::Exact(vec![DataType::Int64]),
                    TypeSignature::Exact(vec![DataType::Int64, DataType::Int64]),
                ],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for QuadBinToParent {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for QuadBinToParent {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "quadbin_to_parent"
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
                "Return a QUADBIN parent cell from a quadbin cell with an optional resolution",
                "quadbin_to_parent(5256690695657226239) or quadbin_to_parent(5256690695657226239,13)",
            )
            .with_argument("cell", "cell value")
            .with_argument("resolution", "resolution value")
            .build()
        }))
    }
}

fn return_field_impl(_args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    Ok(Arc::new(Field::new("", DataType::UInt64, false)))
}

fn build_cell_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<UInt64Array> {
    let cell = arrays[0].as_primitive::<Int64Type>();
    let resolution = arrays.get(1).map(|arr| arr.as_primitive::<Int64Type>());

    let mut builder = UInt64Builder::with_capacity(cell.len());
    match resolution {
        Some(resolution) => {
            for (cell, resolution) in cell.iter().zip(resolution.iter()) {
                if let (Some(cell), Some(resolution)) = (cell, resolution) {
                    let parent = cell_to_parent_resolution(cell as u64, resolution as u8);
                    builder.append_value(parent);
                }
            }
        }
        None => {
            for cell in cell.iter() {
                if let Some(cell) = cell {
                    let parent = cell_to_parent(cell as u64);
                    builder.append_value(parent);
                }
            }
        }
    };

    let point_arr = builder.finish();

    Ok(point_arr)
}

// #[cfg(test)]
// mod tests {  
//     use datafusion::prelude::SessionContext;  

//     use super::*;

//     #[tokio::test]
//     async fn test_quadbin_to_parent() {
//         let ctx = SessionContext::new();
//         ctx.register_udf(QuadBinToParent::default().into());
//         let sql = r#"SELECT quadbin_to_parent(5256690695657226239) cell;"#;
//         println!("{:?}", sql);

//         let df = ctx.sql(sql).await.unwrap();       
//         let batches = df.collect().await.unwrap();
//         let column = batches[0].column(0);
//         // let string_arr = column.as_string_view();

//         let val = column.as_primitive::<UInt64Type>().value(0);
//         println!("{:?}", val);

//     }

//     #[tokio::test]
//     async fn test_quadbin_to_parent_resolution() {
//         let ctx = SessionContext::new();
//         ctx.register_udf(QuadBinToParent::default().into());
//         let sql = r#"SELECT quadbin_to_parent(5256690695657226239,13) cell;"#;
//         println!("{:?}", sql);

//         let df = ctx.sql(sql).await.unwrap();       
//         let batches = df.collect().await.unwrap();
//         let column = batches[0].column(0);
//         // let string_arr = column.as_string_view();

//         let val = column.as_primitive::<UInt64Type>().value(0);
//         println!("{:?}", val);

//     }
// }
