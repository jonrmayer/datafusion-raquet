use std::any::Any;
use std::sync::{Arc, OnceLock};

use arrow_array::builder::UInt64Builder;
use arrow_array::cast::AsArray;
use arrow_array::types::{Float64Type, Int64Type};
use arrow_array::{ArrayRef, UInt64Array};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

use crate::error::RaquetDataFusionResult;

use quadbin_rs::QuadBin;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct QuadBinFromLonLat {
    signature: Signature,
}

impl QuadBinFromLonLat {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(
                vec![DataType::Float64, DataType::Float64, DataType::Int64],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for QuadBinFromLonLat {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for QuadBinFromLonLat {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "quadbin_from_lonlat"
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
                "Return a QUADBIN cell from a LonLat and Resolution(lon,lat,resolution).",
                "quadbin_from_latlon(lon,lat,resolution)",
            )
            .with_argument("lon", "lon value")
            .with_argument("lat", "lat value")
            .with_argument("resolution", "resolution value")
            .build()
        }))
    }
}

fn return_field_impl(_args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    Ok(Arc::new(Field::new("", DataType::UInt64, false)))
}

fn build_cell_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<UInt64Array> {
    let lon = arrays[0].as_primitive::<Float64Type>();
    let lat = arrays[1].as_primitive::<Float64Type>();
    let resolution = arrays[2].as_primitive::<Int64Type>();

    let mut builder = UInt64Builder::with_capacity(lon.len());

    for ((lon, lat), resolution) in lon.iter().zip(lat.iter()).zip(resolution.iter()) {
        // let tile: Tile = Tile::new(x.unwrap() as u32, y.unwrap() as u32, z.unwrap() as i8).unwrap();
        let cell = QuadBin::from_lonlat(lon.unwrap(), lat.unwrap(), resolution.unwrap() as i8)?.cell();
        builder.append_value(cell);
    }
    let point_arr = builder.finish();

    Ok(point_arr)
}

// #[cfg(test)]
// mod tests {

//     use datafusion::prelude::SessionContext;

//     use super::*;

//     #[tokio::test]
//     async fn test_quadbin_from_tile() {
//         let ctx = SessionContext::new();
//         ctx.register_udf(QuadBinFromLonLat::default().into());

//         let sql = r#"SELECT quadbin_from_lonlat(0.0, 0.0, 5);"#;
//         println!("{:?}", sql);

//         let df = ctx.sql(sql).await.unwrap();

//         let schema = df.schema().clone();
//         let batches = df.collect().await.unwrap();
//         let column = batches[0].column(0);
//         // let string_arr = column.as_string_view();

//         let val = column.as_primitive::<UInt64Type>().value(0);
//         println!("{:?}", val);

     
//     }
// }
