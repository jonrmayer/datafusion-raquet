use std::any::Any;
use std::sync::{Arc, OnceLock};

use crate::error::RaquetDataFusionResult;
use arrow_array::builder::{BinaryViewBuilder,GenericBinaryBuilder};
use arrow_array::{ArrayRef, BinaryArray, BinaryViewArray, LargeBinaryArray};
use arrow_schema::{DataType, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    TypeSignature, Volatility,
};

use rastertile_schema::{Metadata, RasterArrowType, RasterType};

use rastertile_rs::{CompressionFormat, Operations};

use crate::udf::raster::utils::has_extension;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct DecompressTile {
    signature: Signature,
}

impl DecompressTile {
    pub fn new() -> Self {
        Self {
            signature: Signature::one_of(
                vec![
                    TypeSignature::Exact(vec![DataType::Binary]),
                    TypeSignature::Exact(vec![DataType::LargeBinary]),
                    TypeSignature::Exact(vec![DataType::BinaryView]),
                ],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for DecompressTile {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for DecompressTile {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn name(&self) -> &str {
        "raquet_decompress"
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
                let col_value = build_cell_array(arrays, binary_type, column_metadata)?;

                Ok(col_value)
            }
            false => Err(DataFusionError::Internal("invoke_with_args".to_string())),
        }
    }

    fn documentation(&self) -> Option<&Documentation> {
        Some(DOCUMENTATION.get_or_init(|| {
            Documentation::builder(
                DOC_SECTION_OTHER,
                "Return a decompressed binary from an compressed binary.",
                "raquet_band_decompress(band_1,metadata)",
            )
            .with_argument("band", "band value")
            .with_argument("metadata", "optional metadata value")
            .build()
        }))
    }
}

fn return_field_impl(args: ReturnFieldArgs) -> Result<FieldRef> {
    let binary_field = &args.arg_fields[0];
    match has_extension(args.arg_fields[0].as_ref()) {
        true => {
            let existing_metadata = Metadata::try_from(binary_field.as_ref()).unwrap_or_default();
            let new_metadata = Metadata::new(
                existing_metadata.tile_size(),
                existing_metadata.binary_type(),
                existing_metadata.data_type(),
                existing_metadata.no_data(),
                CompressionFormat::None,
                existing_metadata.bands(),
            );
            let metadata = Arc::new(new_metadata);
            let raster_type = RasterType::new(metadata);
            let raster_arrow_type = match binary_field.data_type() {
                DataType::Binary => RasterArrowType::Raster(raster_type),
                DataType::LargeBinary => RasterArrowType::LargeRaster(raster_type),
                DataType::BinaryView => RasterArrowType::RasterView(raster_type),
                _ => unreachable!(),
            };
            Ok(Arc::new(raster_arrow_type.to_field("", true)))
        }
        false => Err(DataFusionError::Internal("return_field_impl".to_string())),
    }
}

fn build_cell_array(
    arrays: Vec<ArrayRef>,
    binary_type: &DataType,
    column_metadata: Metadata,
) -> RaquetDataFusionResult<ColumnarValue> {
    
    match binary_type {
        DataType::Binary => {
            let mut builder = GenericBinaryBuilder::<i32>::new();
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryArray>()
                .expect("cast failed");
            let ops: Operations = Operations::new(column_metadata.inner());
            for input in in_binary.iter() {
                let output = ops.decompress(input)?;

                builder.append_value(output);
            }
            let point_arr = builder.finish();
            Ok(ColumnarValue::Array(Arc::new(point_arr)))
           
        }
        DataType::LargeBinary => {
            let mut builder = GenericBinaryBuilder::<i64>::new();
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<LargeBinaryArray>()
                .expect("cast failed");
            let ops: Operations = Operations::new(column_metadata.inner());
            for input in in_binary.iter() {
                let output = ops.decompress(input)?;

                builder.append_value(output);
            }
            let point_arr = builder.finish();
            Ok(ColumnarValue::Array(Arc::new(point_arr)))
        }

        DataType::BinaryView => {
            let mut builder = BinaryViewBuilder::new();
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryViewArray>()
                .expect("cast failed");
            let ops: Operations = Operations::new(column_metadata.inner());
            for input in in_binary.iter() {
                let output = ops.decompress(input)?;

                builder.append_value(output);
            }
            let point_arr = builder.finish();
            Ok(ColumnarValue::Array(Arc::new(point_arr)))
        }
        _ => unreachable!(),
    }
}
