use std::any::Any;
use std::sync::{Arc, OnceLock};

use crate::error::RaquetDataFusionResult;

use arrow_array::builder::Float64Builder;
use arrow_array::cast::AsArray;
use arrow_array::cast::as_string_array;
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

use quadbin_rs::lonlat_to_pixel;

use quadbin_geo_rs::wkt_to_lonlat;

use rastertile_rs::Operations;

use crate::udf::raster::utils::has_extension;
use rastertile_schema::Metadata;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct RaquetValue {
    signature: Signature,
}

impl RaquetValue {
    pub fn new() -> Self {
        Self {
            signature: Signature::one_of(
                vec![
                    TypeSignature::Exact(vec![DataType::Binary, DataType::Utf8, DataType::Int64]),
                    TypeSignature::Exact(vec![
                        DataType::BinaryView,
                        DataType::Utf8,
                        DataType::Int64,
                    ]),
                    TypeSignature::Exact(vec![
                        DataType::LargeBinary,
                        DataType::Utf8,
                        DataType::Int64,
                    ]),
                ],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for RaquetValue {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for RaquetValue {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn name(&self) -> &str {
        "raquet_value"
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
                "Return a decoded binary   an encoded binary.",
                "raquet_value(band_1,wkt,resolution)",
            )
            .with_argument("band", "band value")
            .with_argument("wkt", "wkt value")
            .with_argument("resolution", "resolution value")
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
    let wkt_array = as_string_array(&arrays[1]);
    let resolution_array = arrays[2].as_primitive::<Int64Type>();

    match binary_type {
        DataType::Binary => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryArray>()
                .expect("cast failed");
            for (binary, wkt, resolution) in multizip((in_binary, wkt_array, resolution_array)) {
                let lonlat = wkt_to_lonlat(wkt.unwrap().to_string());

                let pixel = lonlat_to_pixel(
                    lonlat.0,
                    lonlat.1,
                    resolution.unwrap() as i8,
                    column_metadata.inner().tile_size() as i16,
                );

                let value = ops.getpixel(binary, pixel.pixel_x as u64, pixel.pixel_y as u64)?;
                out_builder.append_value(value);
            }
        }
        DataType::LargeBinary => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<LargeBinaryArray>()
                .expect("cast failed");
            for (binary, wkt, resolution) in multizip((in_binary, wkt_array, resolution_array)) {
                let lonlat = wkt_to_lonlat(wkt.unwrap().to_string());

                let pixel = lonlat_to_pixel(
                    lonlat.0,
                    lonlat.1,
                    resolution.unwrap() as i8,
                    column_metadata.inner().tile_size() as i16,
                );

                let value = ops.getpixel(binary, pixel.pixel_x as u64, pixel.pixel_y as u64)?;
                out_builder.append_value(value);
            }
        }

        DataType::BinaryView => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryViewArray>()
                .expect("cast failed");
            for (binary, wkt, resolution) in multizip((in_binary, wkt_array, resolution_array)) {
                let lonlat = wkt_to_lonlat(wkt.unwrap().to_string());

                let pixel = lonlat_to_pixel(
                    lonlat.0,
                    lonlat.1,
                    resolution.unwrap() as i8,
                    column_metadata.inner().tile_size() as i16,
                );

                let value = ops.getpixel(binary, pixel.pixel_x as u64, pixel.pixel_y as u64)?;
                out_builder.append_value(value);
            }
        }
        _ => unreachable!(),
    }

    let point_arr = out_builder.finish();

    Ok(point_arr)
}
