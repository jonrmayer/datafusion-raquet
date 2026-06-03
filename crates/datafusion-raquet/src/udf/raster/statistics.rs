use std::any::Any;
use std::sync::{Arc, OnceLock};

use crate::error::RaquetDataFusionResult;
use arrow_array::builder::UInt64Builder;
use arrow_array::types::UInt64Type;
use arrow_array::{ArrayRef, BinaryArray, PrimitiveArray};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

use rasterarrow_schema::Metadata;

use rastertile_rs::{
    CompressionFormat, NewDataType, Tile,
};

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct StatisticsTile {
    signature: Signature,
}

impl StatisticsTile {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(vec![DataType::Binary], Volatility::Immutable),
        }
    }
}

impl Default for StatisticsTile {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for StatisticsTile {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "statistics_tile"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        Err(DataFusionError::Internal("return_type".to_string()))
    }

    fn return_field_from_args(&self, _args: ReturnFieldArgs) -> Result<FieldRef> {
        Ok(Arc::new(Field::new("", DataType::UInt64, false)))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        // let field = &args.arg_fields[0];
        // let to_type = RasterArrowType::from_arrow_field(args.return_field.as_ref()).unwrap();
        let existing_metadata = Metadata::try_from(args.arg_fields[0].as_ref()).unwrap_or_default();
        let arrays = ColumnarValue::values_to_arrays(&args.args)?;
        let cell_arr = build_cell_array(arrays, existing_metadata)?;
        let array_ref: ArrayRef = Arc::new(cell_arr);
        Ok(ColumnarValue::Array(array_ref))
    }

    fn documentation(&self) -> Option<&Documentation> {
        Some(DOCUMENTATION.get_or_init(|| {
            Documentation::builder(
                DOC_SECTION_OTHER,
                "Return a decoded binary from an encoded binary.",
                "decode_tile(tile)",
            )
            .with_argument("tile", "tile value")
            .build()
        }))
    }
}

fn return_field_impl(_args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    let lf = Field::new_list_field(DataType::Float32, true);
    let dt = DataType::List(Arc::new(lf));
    let out_field: Field = Field::new("", dt, true);
    // let input_field = &args.arg_fields[0];
    // let existing_metadata = Arc::new(Metadata::try_from(input_field.as_ref())?);
    // let new_metadata = Metadata::new(
    //     existing_metadata.tile_size,
    //     existing_metadata.binary_type,
    //     existing_metadata.data_type,
    //     CompressionFormat::None,
    // );
    // let metadata = Arc::new(new_metadata);
    // let values_field: Field = Field::new("", DataType::Float32, true);
    // let dt = DataType::List(Arc::new(values_field));
    // Field::
    // let out_field: Field = Field::
    // let output_type = RasterFloat32Type::new(metadata);
    Ok(Arc::new(out_field))
    // Ok(raster_type
    //     .to_field(input_field.name(), input_field.is_nullable())
    //     .into())
}
fn convert(_metadata: Metadata, data: Option<&[u8]>) -> u64 {
    let tile: Tile = Tile {
        x: 256,
        y: 256,
        data_type: Some(NewDataType::Float32),
        compressed_bytes: data.unwrap().to_vec(),
        compression_method: CompressionFormat::Gzip,
    };

    let ts = tile.statistics().unwrap();

    ts.valid_count
}

fn build_cell_array(
    arrays: Vec<ArrayRef>,
    metadata: Metadata,
) -> RaquetDataFusionResult<PrimitiveArray<UInt64Type>> {
    let in_binary = arrays[0]
        .as_any()
        .downcast_ref::<BinaryArray>()
        .expect("cast failed");

    let mut values_builder = UInt64Builder::new();
    // let mut builder = StructBuilder::new(values_builder);

    for input in in_binary.iter() {
        let output = convert(metadata.clone(), input);
        values_builder.append_value(output);
    }

    let point_arr = values_builder.finish();

    Ok(point_arr)
}
