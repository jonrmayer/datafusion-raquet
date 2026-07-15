use std::any::Any;
use std::sync::{Arc, OnceLock};

use crate::error::RaquetDataFusionResult;

use arrow_array::{ArrayRef, BinaryArray, BinaryViewArray, LargeBinaryArray};
use arrow_schema::{DataType, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    TypeSignature, Volatility,
};

use datafusion::common::ScalarValue;

use rastertile_schema::{Metadata, RasterArrowType, RasterType};

use rastertile_rs::Metadata as RasterMetadata;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct CastRaquet {
    signature: Signature,
}

impl CastRaquet {
    pub fn new() -> Self {
        Self {
            signature: Signature::one_of(
                vec![
                    TypeSignature::Exact(vec![
                        DataType::Binary,
                        DataType::Utf8,
                        DataType::Utf8,
                        DataType::Utf8,
                        DataType::Utf8,
                        DataType::Utf8,
                    ]),
                    TypeSignature::Exact(vec![
                        DataType::BinaryView,
                        DataType::Utf8,
                        DataType::Utf8,
                        DataType::Utf8,
                        DataType::Utf8,
                        DataType::Utf8,
                    ]),
                    TypeSignature::Exact(vec![
                        DataType::LargeBinary,
                        DataType::Utf8,
                        DataType::Utf8,
                        DataType::Utf8,
                        DataType::Utf8,
                        DataType::Utf8,
                    ]),
                ],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for CastRaquet {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for CastRaquet {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn name(&self) -> &str {
        "binary_to_raquet"
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
        let colvalue = build_cell_array(arrays, binary_type)?;

        Ok(colvalue)
    }

    fn documentation(&self) -> Option<&Documentation> {
        Some(DOCUMENTATION.get_or_init(|| {
            Documentation::builder(
                DOC_SECTION_OTHER,
                "Return a raquet binary from a binary.",
                "binary_to_raquet(band_1,'256', 'Separated', 'float32','NaN','gzip')",
            )
            .with_argument("band", "band value")
            .with_argument("tile_size", "tile_size value")
            .with_argument("binary_type", "binary_type value")
            .with_argument("data_type", "data_type value")
            .with_argument("no_data", "no_data value")
            .with_argument("compression", "compression value")
            .build()
        }))
    }
}

fn return_field_impl(args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    let binary_field = &args.arg_fields[0];

    let scalar_args = args.scalar_arguments;

    let tile_size_str = if let Some(ScalarValue::Utf8(Some(tile_size))) = scalar_args[1] {
        tile_size.clone()
    } else {
        "".to_string()
    };
    let binary_type_str = if let Some(ScalarValue::Utf8(Some(binary_type))) = scalar_args[2] {
        binary_type.clone()
    } else {
        "".to_string()
    };
    let data_type_str = if let Some(ScalarValue::Utf8(Some(data_type))) = scalar_args[3] {
        data_type.clone()
    } else {
        "".to_string()
    };
    let no_data_str = if let Some(ScalarValue::Utf8(Some(no_data))) = scalar_args[4] {
        no_data.clone()
    } else {
        "".to_string()
    };
    let compression_str = if let Some(ScalarValue::Utf8(Some(compression))) = scalar_args[5] {
        compression.clone()
    } else {
        "".to_string()
    };

    let rm = RasterMetadata::new_from_strings(
        tile_size_str,
        binary_type_str,
        data_type_str,
        no_data_str,
        compression_str,
        None,
    );
    let new_metadata = Metadata::new(
        rm.tile_size(),
        rm.binary_type(),
        rm.data_type(),
        rm.no_data(),
        rm.compression(),
        rm.bands(),
    );
    let metadata = Arc::new(new_metadata);
    // GeoArrowType:
    let raster_type = RasterType::new(metadata);
    let raster_arrow_type = match binary_field.data_type() {
        DataType::Binary => RasterArrowType::Raster(raster_type),
        DataType::LargeBinary => RasterArrowType::LargeRaster(raster_type),
        DataType::BinaryView => RasterArrowType::RasterView(raster_type),
        _ => unreachable!(),
    };

    Ok(Arc::new(
        raster_arrow_type.to_field(binary_field.name(), true),
    ))
}

fn build_cell_array(
    arrays: Vec<ArrayRef>,
    binary_type: &DataType,
) -> RaquetDataFusionResult<ColumnarValue> {
    match binary_type {
        DataType::Binary => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryArray>()
                .expect("cast failed");
            Ok(ColumnarValue::Array(Arc::new(in_binary.clone())))
        }
        DataType::LargeBinary => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<LargeBinaryArray>()
                .expect("cast failed");
            Ok(ColumnarValue::Array(Arc::new(in_binary.clone())))
        }

        DataType::BinaryView => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryViewArray>()
                .expect("cast failed");
            Ok(ColumnarValue::Array(Arc::new(in_binary.clone())))
        }
        _ => unreachable!(),
    }
}
