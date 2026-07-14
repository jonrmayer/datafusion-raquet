use std::sync::{Arc, OnceLock};
use std::any::Any;

use arrow_array::builder::{ StringBuilder, Int32Builder};
use arrow_array::cast::as_string_array;

use arrow_array::{ArrayRef,  StructArray};
use arrow_schema::{DataType, Field, FieldRef, Fields};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

use crate::error::RaquetDataFusionResult;

use crate::{raquet_band_metadata};


#[derive(Debug, Eq, PartialEq, Hash)]
pub struct BandMetadata {
    signature: Signature,
}

impl BandMetadata {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(
                vec![DataType::Utf8, DataType::Utf8],
                Volatility::Immutable,
            ),
        }
    }
    fn data_type(&self) -> DataType {
        let values_fields = vec![
            Field::new("tile_size", DataType::Int32, false),
            Field::new("binary_type", DataType::Utf8, false),
            Field::new("data_type", DataType::Utf8, false),
            Field::new("no_data", DataType::Utf8, false),
            Field::new("compression", DataType::Utf8, false),
        ];
        DataType::Struct(values_fields.into())
    }
    fn to_field<N: Into<String>>(&self, name: N, nullable: bool) -> Field {
        Field::new(name, self.data_type(), nullable)
    }
}

impl Default for BandMetadata {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for BandMetadata {
     fn as_any(&self) -> &dyn Any {
        self
    }
    fn name(&self) -> &str {
        "band_metadata"
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
                "band_metadata(band_name,metadata)",
            )
            .with_argument("band_name", "band_name value")
            .with_argument("metadata", "metadata value")
            .build()
        }))
    }
}

fn build_cell_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<StructArray> {
    let band_name_array = as_string_array(&arrays[0]);
    let metadata_array = as_string_array(&arrays[1]);

    let mut tile_size_builder = Int32Builder::new();
    let mut binary_type_builder = StringBuilder::new();
    let mut data_type_builder = StringBuilder::new();
    let mut no_data_builder = StringBuilder::new();
    let mut compression_builder = StringBuilder::new();

    for (metadata, band_name) in metadata_array.iter().zip(band_name_array.iter()) {
        let rcm = raquet_band_metadata(band_name.unwrap(), metadata.unwrap());

        tile_size_builder.append_value(rcm.tile_size as i32);
        binary_type_builder.append_value(rcm.binary_type.to_string());
        data_type_builder.append_value(rcm.data_type.to_string());
        no_data_builder.append_value(rcm.no_data);
        compression_builder.append_value(rcm.compression.to_string());
    }

    let values_fields = vec![
        Field::new("tile_size", DataType::Int32, false),
        Field::new("binary_type", DataType::Utf8, false),
        Field::new("data_type", DataType::Utf8, false),
        Field::new("no_data", DataType::Utf8, false),
        Field::new("compression", DataType::Utf8, false),
    ];

    let fields = Fields::from(values_fields);

    let arrays: Vec<ArrayRef> = vec![
        Arc::new(tile_size_builder.finish()),
        Arc::new(binary_type_builder.finish()),
        Arc::new(data_type_builder.finish()),
        Arc::new(no_data_builder.finish()),
        Arc::new(compression_builder.finish()),
    ];
    let nulls = None;
    let arr = StructArray::new(fields, arrays, nulls);

    Ok(arr)

}


