use arrow_array::builder::Float64Builder;
use arrow_array::cast::AsArray;
use arrow_array::types::UInt64Type;
use arrow_array::{ArrayRef, StructArray};
use arrow_schema::{DataType, Field, FieldRef, Fields};
use std::any::Any;
use std::sync::{Arc, OnceLock};

use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

use crate::error::RaquetDataFusionResult;

use quadbin_rs::QuadBin;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct QuadBinToLonLat {
    signature: Signature,
}

impl QuadBinToLonLat {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(vec![DataType::UInt64], Volatility::Immutable),
        }
    }
    fn data_type(&self) -> DataType {
        let values_fields = vec![
            Field::new("lon", DataType::Float64, false),
            Field::new("lat", DataType::Float64, false),
        ];
        DataType::Struct(values_fields.into())
    }
    fn to_field<N: Into<String>>(&self, name: N, nullable: bool) -> Field {
        Field::new(name, self.data_type(), nullable)
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

    fn return_field_from_args(&self, _args: ReturnFieldArgs) -> Result<FieldRef> {
        Ok(Arc::new(self.to_field("", false)))
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

fn build_cell_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<StructArray> {
    let cells = arrays[0].as_primitive::<UInt64Type>();
    let mut lon_builder = Float64Builder::new();
    let mut lat_builder = Float64Builder::new();

    for cell in cells.iter() {
        let lonlat = QuadBin::from_cell(cell.unwrap())?.to_lonlat()?;
        lon_builder.append_value(lonlat.0);
        lat_builder.append_value(lonlat.1);
    }

    let values_fields = vec![
        Field::new("lon", DataType::Float64, false),
        Field::new("lat", DataType::Float64, false),
    ];

    let fields = Fields::from(values_fields);

    let arrays: Vec<ArrayRef> = vec![
        Arc::new(lon_builder.finish()),
        Arc::new(lat_builder.finish()),
    ];
    let nulls = None;
    let arr = StructArray::new(fields, arrays, nulls);

    Ok(arr)
}
