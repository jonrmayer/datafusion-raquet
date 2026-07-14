use std::any::Any;
use std::sync::{Arc, OnceLock};

use arrow_array::builder::Int32Builder;
use arrow_array::cast::as_string_array;

use arrow_array::{ArrayRef, StructArray};
use arrow_schema::{DataType, Field, FieldRef, Fields};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

use crate::error::RaquetDataFusionResult;

use crate::raquet_quadbin_metadata;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct QuadbinMetadata {
    signature: Signature,
}

impl QuadbinMetadata {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(vec![DataType::Utf8], Volatility::Immutable),
        }
    }

    fn data_type(&self) -> DataType {
        let values_fields = vec![
            Field::new("min_zoom", DataType::Int32, false),
            Field::new("max_zoom", DataType::Int32, false),
        ];
        DataType::Struct(values_fields.into())
    }
    fn to_field<N: Into<String>>(&self, name: N, nullable: bool) -> Field {
        Field::new(name, self.data_type(), nullable)
    }
}

impl Default for QuadbinMetadata {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for QuadbinMetadata {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn name(&self) -> &str {
        "quadbin_metadata"
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
                "Return a metdata struct for a band",
                "quadbin_metadata(metadata)",
            )
            .with_argument("metadata", "metadata value")
            .build()
        }))
    }
}

fn build_cell_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<StructArray> {
    let metadata_array = as_string_array(&arrays[0]);

    let mut min_zoom_builder = Int32Builder::new();
    let mut max_zoom_builder = Int32Builder::new();

    for metadata in metadata_array.iter() {
        let rqm = raquet_quadbin_metadata(metadata.unwrap());

        min_zoom_builder.append_value(rqm.min_zoom);
        max_zoom_builder.append_value(rqm.max_zoom);
    }

    let values_fields = vec![
        Field::new("min_zoom", DataType::Int32, false),
        Field::new("max_zoom", DataType::Int32, false),
    ];

    let fields = Fields::from(values_fields);

    let arrays: Vec<ArrayRef> = vec![
        Arc::new(min_zoom_builder.finish()),
        Arc::new(max_zoom_builder.finish()),
    ];
    let nulls = None;
    let arr = StructArray::new(fields, arrays, nulls);

    Ok(arr)
}
