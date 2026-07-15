use std::any::Any;
use std::sync::{Arc, OnceLock};

use crate::error::RaquetDataFusionResult;

use arrow_array::builder::Float64Builder;
use arrow_array::cast::AsArray;
use arrow_array::types::Int64Type;
use arrow_array::{ArrayRef, BinaryArray, BinaryViewArray, Float64Array, LargeBinaryArray};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    TypeSignature, Volatility,
};

use itertools::multizip;

use rastertile_schema::Metadata;

use crate::udf::raster::utils::has_extension;
use rastertile_rs::Operations;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct RaquetPixel {
    signature: Signature,
}

impl RaquetPixel {
    pub fn new() -> Self {
        Self {
            // signature: Signature::exact(
            //     vec![DataType::Binary, DataType::Int64, DataType::Int64],
            //     Volatility::Immutable,
            // ),
            signature: Signature::one_of(
                vec![
                    TypeSignature::Exact(vec![DataType::Binary, DataType::Int64, DataType::Int64]),
                    TypeSignature::Exact(vec![
                        DataType::BinaryView,
                        DataType::Int64,
                        DataType::Int64,
                    ]),
                    TypeSignature::Exact(vec![
                        DataType::LargeBinary,
                        DataType::Int64,
                        DataType::Int64,
                    ]),
                ],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for RaquetPixel {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for RaquetPixel {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn name(&self) -> &str {
        "raquet_pixel"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        Err(DataFusionError::Internal("return_type".to_string()))
    }

    fn return_field_from_args(&self, _args: ReturnFieldArgs) -> Result<FieldRef> {
        Ok(Arc::new(Field::new("", DataType::Float64, false)))
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
                "Return a decoded binary from an encoded binary.",
                "raquet_pixel(band,pixel_x,pixel_y)",
            )
            .with_argument("band", "band value")
            .with_argument("pixel_x", "pixel_x value")
            .with_argument("pixel_y", "pixel_y value")
            .build()
        }))
    }
}

fn build_cell_array(
    arrays: Vec<ArrayRef>,
    binary_type: &DataType,
    column_metadata: Metadata,
) -> RaquetDataFusionResult<Float64Array> {
    let mut out_builder = Float64Builder::new();
    let ops: Operations = Operations::new(column_metadata.inner());
    let pixel_x_array = arrays[1].as_primitive::<Int64Type>();
    let pixel_y_array = arrays[2].as_primitive::<Int64Type>();

    match binary_type {
        DataType::Binary => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryArray>()
                .expect("cast failed");
            for (binary, pixel_x, pixel_y) in multizip((in_binary, pixel_x_array, pixel_y_array)) {
                let value =
                    ops.getpixel(binary, pixel_x.unwrap() as u64, pixel_y.unwrap() as u64)?;
                out_builder.append_value(value);
            }
        }
        DataType::LargeBinary => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<LargeBinaryArray>()
                .expect("cast failed");
            for (binary, pixel_x, pixel_y) in multizip((in_binary, pixel_x_array, pixel_y_array)) {
                let value =
                    ops.getpixel(binary, pixel_x.unwrap() as u64, pixel_y.unwrap() as u64)?;
                out_builder.append_value(value);
            }
        }

        DataType::BinaryView => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryViewArray>()
                .expect("cast failed");
            for (binary, pixel_x, pixel_y) in multizip((in_binary, pixel_x_array, pixel_y_array)) {
                let value =
                    ops.getpixel(binary, pixel_x.unwrap() as u64, pixel_y.unwrap() as u64)?;
                out_builder.append_value(value);
            }
        }
        _ => unreachable!(),
    }

    let point_arr = out_builder.finish();

    Ok(point_arr)
}
