use std::any::Any;
use std::sync::{Arc, OnceLock};

use arrow_array::builder::StringViewBuilder;
use arrow_array::cast::AsArray;
use arrow_array::types::{Int64Type, UInt8Type, UInt32Type, UInt64Type};
use arrow_array::{Array, ArrayRef, GenericListArray, ListArray, StructArray, UInt64Array};
use arrow_schema::{DataType, Field, FieldRef, Fields};

// use arrow_convert::{
//     ArrowDeserialize, ArrowField, ArrowSerialize, deserialize::TryIntoCollection,
//     serialize::TryIntoArrow,
// };

use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    TypeSignature, Volatility,
};

use crate::error::{RaquetDataFusionError, RaquetDataFusionResult};

// use crate::udf::quadbin::converter::{Abbox, LonLat, Pixel};
use quadbin_geo_rs::GeoFormats;
use quadbin_rs::QuadBin;

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
        let bbox = QuadBin::from_cell(cell.unwrap() as u64)?
            .to_tile()?
            .to_bbox_wgs84()?;
        let wkt = GeoFormats::new(bbox).to_wkt();
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
        let sql = r#"SELECT quadbin_to_wkt(5202642732031410175) ;"#;
        println!("{:?}", sql);

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }
}
