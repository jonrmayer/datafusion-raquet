use arrow_array::ArrayRef;
use arrow_array::builder::StringViewBuilder;
use arrow_array::cast::AsArray;
use arrow_array::types::UInt64Type;
use arrow_schema::{DataType, Field, FieldRef};
use std::any::Any;
use std::sync::{Arc, OnceLock};

use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};
use quadbin_geo_rs::GeoFormats;

use crate::error::RaquetDataFusionResult;

use quadbin_rs::QuadBin;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct QuadBinToGeoJSON {
    signature: Signature,
}

impl QuadBinToGeoJSON {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(vec![DataType::UInt64], Volatility::Immutable),
        }
    }
}

impl Default for QuadBinToGeoJSON {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for QuadBinToGeoJSON {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn name(&self) -> &str {
        "quadbin_to_geojson"
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
        let cell_arr = build_geojson_array(arrays)?;
        Ok(cell_arr)
    }

    fn documentation(&self) -> Option<&Documentation> {
        Some(DOCUMENTATION.get_or_init(|| {
            Documentation::builder(
                DOC_SECTION_OTHER,
                "Return a GeoJSON String from a quadbin cell ",
                "quadbin_to_geojson(5256690695657226239) ",
            )
            .with_argument("cell", "cell value")
            .build()
        }))
    }
}

fn return_field_impl(_args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    Ok(Arc::new(Field::new("", DataType::Utf8View, false)))
}

fn build_geojson_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<ColumnarValue> {
    let cells = arrays[0].as_primitive::<UInt64Type>();
    let mut builder = StringViewBuilder::with_capacity(cells.len());

    for cell in cells.iter() {
        let bbox = QuadBin::from_cell(cell.unwrap())?
            .to_tile()?
            .to_bbox_wgs84()?;
        let geojson = GeoFormats::new(bbox).to_geojson();
        builder.append_value(geojson);
    }

    Ok(ColumnarValue::Array(Arc::new(builder.finish())))
}

