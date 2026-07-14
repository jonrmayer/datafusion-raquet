use std::any::Any;
use std::sync::{Arc, OnceLock};

use arrow_array::ArrayRef;
use arrow_array::builder::StringViewBuilder;
use arrow_array::cast::AsArray;
use arrow_array::types::UInt64Type;
use arrow_schema::{DataType, Field, FieldRef};

use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

use crate::error::RaquetDataFusionResult;

use quadbin_geo_rs::GeoFormats;
use quadbin_rs::QuadBin;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct QuadBinToWKT {
    signature: Signature,
}

impl QuadBinToWKT {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(vec![DataType::UInt64], Volatility::Immutable),
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
    let cells = arrays[0].as_primitive::<UInt64Type>();
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
