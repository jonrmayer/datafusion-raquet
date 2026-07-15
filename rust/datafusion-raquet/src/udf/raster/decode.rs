use std::any::Any;
use std::sync::{Arc, OnceLock};

use crate::error::RaquetDataFusionResult;
use arrow_array::builder::{Float64Builder, ListBuilder};

use arrow_array::{ArrayRef, BinaryArray, BinaryViewArray, LargeBinaryArray, ListArray};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    TypeSignature, Volatility,
};

use rastertile_schema::Metadata;

use rastertile_rs::Operations;

use crate::udf::raster::utils::has_extension;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct DecodeTile {
    signature: Signature,
}

impl DecodeTile {
    pub fn new() -> Self {
        Self {
            signature: Signature::one_of(
                vec![
                    TypeSignature::Exact(vec![DataType::Binary]),
                    TypeSignature::Exact(vec![DataType::BinaryView]),
                    TypeSignature::Exact(vec![DataType::LargeBinary]),
                ],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for DecodeTile {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for DecodeTile {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn name(&self) -> &str {
        "raquet_band_decode"
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
        let binary_field = &args.arg_fields[0];
        let binary_type = binary_field.data_type();
        let arrays = ColumnarValue::values_to_arrays(&args.args)?;

        match has_extension(binary_field.as_ref()) {
            true => {
                let column_metadata = Metadata::try_from(binary_field.as_ref()).unwrap_or_default();
                let list_array = build_cell_array(arrays, binary_type, column_metadata)?;

                Ok(ColumnarValue::Array(Arc::new(list_array)))
            }
            false => Err(DataFusionError::Internal("invoke_with_args".to_string())),
        }
    }

    fn documentation(&self) -> Option<&Documentation> {
        Some(DOCUMENTATION.get_or_init(|| {
            Documentation::builder(
                DOC_SECTION_OTHER,
                "Decode band bytes to []Float64 .",
                "raquet_band_decode(band)",
            )
            .with_argument("band", "band value")
            .build()
        }))
    }
}

fn return_field_impl(_args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    let list_field = Field::new_list_field(DataType::Float64, true);
    let dt = DataType::List(Arc::new(list_field));
    let out_field: Field = Field::new("", dt, true);

    Ok(Arc::new(out_field))
}

fn build_cell_array(
    arrays: Vec<ArrayRef>,
    binary_type: &DataType,
    column_metadata: Metadata,
) -> RaquetDataFusionResult<ListArray> {
    let values_builder = Float64Builder::new();
    let mut builder = ListBuilder::new(values_builder);
    match binary_type {
        DataType::Binary => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryArray>()
                .expect("cast failed");
            let ops: Operations = Operations::new(column_metadata.inner());
            for input in in_binary.iter() {
                let output = ops.decode(input)?;

                builder.append_value(output);
            }
        }
        DataType::LargeBinary => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<LargeBinaryArray>()
                .expect("cast failed");
            let ops: Operations = Operations::new(column_metadata.inner());
            for input in in_binary.iter() {
                let output = ops.decode(input)?;

                builder.append_value(output);
            }
        }

        DataType::BinaryView => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryViewArray>()
                .expect("cast failed");
            let ops: Operations = Operations::new(column_metadata.inner());
            for input in in_binary.iter() {
                let output = ops.decode(input)?;

                builder.append_value(output);
            }
        }
        _ => unreachable!(),
    }

    let point_arr = builder.finish();

    Ok(point_arr)
}
