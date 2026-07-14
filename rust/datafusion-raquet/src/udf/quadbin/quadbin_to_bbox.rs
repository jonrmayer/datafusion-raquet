use std::sync::{Arc, OnceLock};
use std::any::Any;

use arrow_array::builder::{
     Float64Builder, 
};
use arrow_array::cast::AsArray;
use arrow_array::types::{UInt64Type };
use arrow_array::{ ArrayRef,  StructArray, };
use arrow_schema::{DataType, Field, FieldRef, Fields};



use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    TypeSignature, Volatility,
};

use crate::error::{RaquetDataFusionResult};


use quadbin_rs::QuadBin;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct QuadBinToBBOX {
    signature: Signature,
}

impl QuadBinToBBOX {
    pub fn new() -> Self {
        Self {
            signature: Signature::one_of(
                vec![
                    TypeSignature::Exact(vec![DataType::UInt64]),
                    TypeSignature::Exact(vec![DataType::UInt64, DataType::Int64]),
                ],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for QuadBinToBBOX {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for QuadBinToBBOX {

        fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "quadbin_to_bbox"
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
                "Return a Bbox Struct from a quadbin cell ",
                "quadbin_to_bbox(5202642732031410175) ",
            )
            .with_argument("cell", "cell value")
            .build()
        }))
    }
}

fn return_field_impl(_args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    let min_lon = Field::new("min_lon", DataType::Float64, false);
    let min_lat = Field::new("min_lat", DataType::Float64, false);
    let max_lon = Field::new("max_lon", DataType::Float64, false);
    let max_lat = Field::new("max_lat", DataType::Float64, false);

    let fields = Fields::from(vec![min_lon, min_lat, max_lon, max_lat]);
    let bbox = Field::new_struct("", fields, false);
   
    Ok(Arc::new(bbox))
}

fn build_cell_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<StructArray> {
    let cells = arrays[0].as_primitive::<UInt64Type>();
    let mut xmin_builder = Float64Builder::new();
    let mut ymin_builder = Float64Builder::new();
    let mut xmax_builder = Float64Builder::new();
    let mut ymax_builder = Float64Builder::new();

    for cell in cells.iter() {       
        let bbox_wgs84 = QuadBin::from_cell(cell.unwrap() as u64)?.to_tile()?.to_bbox_wgs84()?;
        xmin_builder.append_value(bbox_wgs84.min_x);
        ymin_builder.append_value(bbox_wgs84.min_y);
        xmax_builder.append_value(bbox_wgs84.max_x);
        ymax_builder.append_value(bbox_wgs84.max_y);
    }

    let values_fields = vec![
        Field::new("min_lon", DataType::Float64, false),
        Field::new("min_lat", DataType::Float64, false),
        Field::new("max_lon", DataType::Float64, false),
        Field::new("max_lat", DataType::Float64, false),
    ];

    let fields = Fields::from(values_fields);

    let arrays: Vec<ArrayRef> = vec![
        Arc::new(xmin_builder.finish()),
        Arc::new(ymin_builder.finish()),
        Arc::new(xmax_builder.finish()),
        Arc::new(ymax_builder.finish()),
    ];
    let nulls = None;
    let arr = StructArray::new(fields, arrays, nulls);
   
     Ok(arr)
}

#[cfg(test)]
mod tests {
    use datafusion::prelude::SessionContext;

    use super::*;

    #[tokio::test]
    async fn test_quadbin_to_children() {
        let ctx = SessionContext::new();
        ctx.register_udf(QuadBinToBBOX::default().into());
        let sql = r#"SELECT quadbin_to_bbox(5202642732031410175) ;"#;
        println!("{:?}", sql);

        let df = ctx.sql(sql).await.unwrap();
       df.show().await.unwrap();
      
    }

   
}
