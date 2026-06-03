use std::any::Any;
use std::sync::{Arc, OnceLock};

use arrow_array::builder::{
    ArrayBuilder, Float64Builder, ListBuilder, StructBuilder, UInt64Builder,
};
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
use quadbin_rs::{cell_to_lonlat};

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct QuadBinToLonLat {
    signature: Signature,
}

impl QuadBinToLonLat {
    pub fn new() -> Self {
         Self {
            signature: Signature::exact(
                vec![DataType::Int64],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for QuadBinToLonLat {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for QuadBinToLonLat {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "quadbin_to_lonlat"
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
                "Return a LONLAT Struct from a quadbin cell ",
                "quadbin_to_lonlat(5256690695657226239) ",
            )
            .with_argument("cell", "cell value")
            .build()
        }))
    }
}

fn return_field_impl(_args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    let lon = Field::new("lon", DataType::Float64, false);
    let lat = Field::new("lat", DataType::Float64, false);
  
    
    let fields = Fields::from(vec![lon,lat]);
    let lonlat = Field::new_struct("", fields, false);
    // let item_field = Arc::new(bbox.clone());
    Ok(Arc::new(lonlat))
}

fn build_cell_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<StructArray> {
    let cells = arrays[0].as_primitive::<Int64Type>();
    let mut vcells: Vec<LonLat> = vec![];
    for cell in cells.iter() {
        let lonlat = cell_to_lonlat(cell.unwrap() as u64);
        let alonlat = LonLat::new(lonlat.0,lonlat.1);
        vcells.push(alonlat);
    }
    let box_array: ArrayRef = vcells.try_into_arrow().unwrap();
    let struct_array = box_array
        .as_any()
        .downcast_ref::<arrow::array::StructArray>()
        .unwrap();
    Ok(struct_array.clone())
}

#[cfg(test)]
mod tests {
    use datafusion::prelude::SessionContext;

    use super::*;

    #[tokio::test]
    async fn test_quadbin_to_children() {
        let ctx = SessionContext::new();
        ctx.register_udf(QuadBinToLonLat::default().into());
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
        ctx.register_udf(QuadBinToLonLat::default().into());
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
