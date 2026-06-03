use std::any::Any;
use std::sync::{Arc, OnceLock};

use arrow_array::builder::UInt8Builder;
use arrow_array::cast::AsArray;
use arrow_array::types::{UInt8Type, UInt32Type, UInt64Type,Int64Type};
use arrow_array::{ArrayRef, UInt64Array,UInt8Array};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};
// use geoarrow_array::GeoArrowArray;
// use geoarrow_array::array::GeometryArray;
// use geoarrow_array::builder::GeometryBuilder;
// use geoarrow_schema::{GeometryType, Metadata};

use crate::error::{RaquetDataFusionError, RaquetDataFusionResult};

use quadbin_rs::{cell_to_resolution};

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct QuadBinResolution {
    signature: Signature,
}

impl QuadBinResolution {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(
                vec![DataType::Int64],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for QuadBinResolution {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for QuadBinResolution {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "quadbin_resolution"
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
                "Return a QUADBIN resolution from a quadbin cell.",
                "quadbin_resolution(5256690695657226239)",
            )
            .with_argument("cell", "cell value")        
            .build()
        }))
    }
}

fn return_field_impl(_args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    Ok(Arc::new(Field::new("", DataType::UInt8, false)))
}



fn build_cell_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<UInt8Array> {
    let cell = arrays[0].as_primitive::<Int64Type>();


    let mut builder = UInt8Builder::with_capacity(cell.len());

    for cell in cell.iter() {      
        let resolution = cell_to_resolution(cell.unwrap() as u64);
        builder.append_value(resolution);
    }
    let point_arr = builder.finish();

    Ok(point_arr)
}


#[cfg(test)]
mod tests {
    // use approx::relative_eq;
    use datafusion::prelude::SessionContext;
    // use geo_traits::{GeometryTrait, LineStringTrait,PolygonTrait};
    // use geoarrow_array::GeoArrowArrayAccessor;
    // use rdf_geoarrow_schema::LineStringType;

    use super::*;

    #[tokio::test]
    async fn test_quadbin_resolution() {
        let ctx = SessionContext::new();
        ctx.register_udf(QuadBinResolution::default().into());

        let sql = r#"SELECT quadbin_resolution(5256690695657226239) cell;"#;
        println!("{:?}", sql);

        let df = ctx.sql(sql).await.unwrap();

        let schema = df.schema().clone();
        let batches = df.collect().await.unwrap();
        let column = batches[0].column(0);
        // let string_arr = column.as_string_view();
        
        let val = column.as_primitive::<UInt8Type>().value(0);
        println!("{:?}", val);

        // let rect_array = GeometryArray::try_from((column.as_ref(), schema.field(0))).unwrap();
        // let rect = rect_array.value(0).unwrap();
        // let poly = match rect.as_type() {
        //     geo_traits::GeometryType::Polygon(poly) => Some(poly),
        //     _ => None,
        // }
        // .unwrap();

        // let b = rect.into()
        // println!("{:?}", poly);

        // assert!(relative_eq!(rect.min().x(), 112.55836486816406));
        // assert!(relative_eq!(rect.min().y(), 37.83236503601074));
        // assert!(relative_eq!(rect.max().x(), 112.5584077835083));
        // assert!(relative_eq!(rect.max().y(), 37.83240795135498));
    }
}
