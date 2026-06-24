use std::any::Any;
use std::sync::{Arc, OnceLock};

use arrow_array::builder::{ListBuilder, UInt64Builder};
use arrow_array::cast::AsArray;
use arrow_array::types::Int64Type;
use arrow_array::{ArrayRef, ListArray, UInt64Array};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature, Volatility,
};

use crate::error::RaquetDataFusionResult;

use quadbin_rs::QuadBin;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct QuadBinToSibling {
    signature: Signature,
}

impl QuadBinToSibling {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(vec![DataType::Int64], Volatility::Immutable),
        }
    }
}

impl Default for QuadBinToSibling {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for QuadBinToSibling {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "quadbin_sibling"
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
                "Return a List[QUADBIN] of cell siblings from a quadbin cell ",
                "quadbin_sibling(5256690695657226239) ",
            )
            .with_argument("cell", "cell value")
            .build()
        }))
    }
}

fn return_field_impl(_args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    let item_field = Arc::new(Field::new("", DataType::UInt64, false));
    Ok(Arc::new(Field::new_list("", item_field.clone(), false)))
}

fn build_cell_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<ListArray> {
    let cell = arrays[0].as_primitive::<Int64Type>();

    let values_builder = UInt64Builder::new();

    let mut builder = ListBuilder::new(values_builder);
    for cell in cell.iter() {
        let cell_siblings = QuadBin::from_cell(cell.unwrap() as u64)?.siblings()?;
        let siblings = UInt64Array::from(cell_siblings);
        builder.append_value(&siblings);
    }

    let point_arr = builder.finish();

    Ok(point_arr)
}

// #[cfg(test)]
// mod tests {
//     use datafusion::prelude::SessionContext;

//     use super::*;

//     #[tokio::test]
//     async fn test_quadbin_to_children() {
//         let ctx = SessionContext::new();
//         ctx.register_udf(QuadBinToSibling::default().into());
//         let sql = r#"SELECT quadbin_sibling(cast(5256690695657226239 as bigint)) cell;"#;
//         println!("{:?}", sql);

//         let df = ctx.sql(sql).await.unwrap();
//         // df.show();
//         let batches = df.collect().await.unwrap();
//         let column = batches[0].column(0);
//         // // let string_arr = column.as_string_view();

//         // let val = column.as_list(0).value(0);
//         // println!("{:?}", val);
//     }

//     #[tokio::test]
//     async fn test_quadbin_to_parent_resolution() {
//         let ctx = SessionContext::new();
//         ctx.register_udf(QuadBinToSibling::default().into());
//         let sql = r#"SELECT quadbin_to_children(5256690695657226239,13) cell;"#;
//         println!("{:?}", sql);

//         let df = ctx.sql(sql).await.unwrap();
//         let batches = df.collect().await.unwrap();
//         let column = batches[0].column(0);
//         // let string_arr = column.as_string_view();

//         let val = column.as_primitive::<UInt64Type>().value(0);
//         println!("{:?}", val);
//     }
// }
