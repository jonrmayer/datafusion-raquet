use std::any::Any;
use std::sync::{Arc, OnceLock};

use crate::error::RaquetDataFusionResult;
use arrow::datatypes::Fields;
use arrow_array::builder::{Float64Builder, UInt64Builder};
use arrow_array::types::UInt64Type;
use arrow_array::{ArrayRef, BinaryArray, PrimitiveArray, StructArray,};
use arrow_array::cast::{AsArray, as_primitive_array, as_string_array};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};
use datafusion_common::scalar::ScalarStructBuilder;
use itertools::multizip;

use rastertile_schema::Metadata;

use rastertile_rs::{CompressionFormat, NewDataType, RasterDataType, Tile, TileStatistics};

use quadbin_geo_rs::wkt_to_lonlat;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct RasterValue {
    signature: Signature,
}

impl RasterValue {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(vec![DataType::Binary,DataType::Utf8], Volatility::Immutable),
        }
    }

    // fn data_type(&self) -> DataType {
    //     let values_fields = vec![
    //         Field::new("min", DataType::Float64, false),
    //         Field::new("max", DataType::Float64, false),
    //         Field::new("mean", DataType::Float64, false),
    //         Field::new("std_dev", DataType::Float64, false),
    //         Field::new("valid_count", DataType::UInt64, false),
    //     ];
    //     DataType::Struct(values_fields.into())
    // }
    // fn to_field<N: Into<String>>(&self, name: N, nullable: bool) -> Field {
    //     Field::new(name, self.data_type(), nullable)
    // }

    // fn builders(&self) {
    //     let b = vec![
    //         Box::new(Float64Builder::new()),
    //         Box::new(Float64Builder::new()),
    //         Box::new(Float64Builder::new()),
    //         Box::new(Float64Builder::new()),
    //         Box::new(UInt64Builder::new()),
    //     ];
    // }
}

impl Default for RasterValue {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for RasterValue {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "raster_value"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        Err(DataFusionError::Internal("return_type".to_string()))
    }

    fn return_field_from_args(&self, _args: ReturnFieldArgs) -> Result<FieldRef> {
        // Ok(Arc::new(self.to_field("", false)))
        Ok(Arc::new(Field::new("", DataType::Float64, false)))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
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

fn get_data_type_from_metadata(metadata: Metadata) -> Option<NewDataType> {
    let data_type: Option<NewDataType> = match metadata.data_type() {
        RasterDataType::UInt8 => Some(NewDataType::UInt8),
        RasterDataType::Int8 => Some(NewDataType::Int8),
        RasterDataType::UInt16 => Some(NewDataType::UInt16),
        RasterDataType::Int16 => Some(NewDataType::Int16),
        RasterDataType::UInt32 => Some(NewDataType::UInt32),
        RasterDataType::Int32 => Some(NewDataType::Int32),
        RasterDataType::UInt64 => Some(NewDataType::UInt64),
        RasterDataType::Int64 => Some(NewDataType::Int64),
        RasterDataType::Float32 => Some(NewDataType::Float32),
        RasterDataType::Float64 => Some(NewDataType::Float64),
    };
    data_type
}

fn convert(metadata: Metadata, data: Option<&[u8]>) -> TileStatistics {
     let samples = match metadata.clone().bands {
        Some(bands) => bands.len(),
        _ => 1,
    };
    let tile: Tile = Tile {
        x: metadata.tile_size().clone(),
        y: metadata.tile_size().clone(),
        data_type: get_data_type_from_metadata(metadata.clone()),
        compressed_bytes: data.unwrap().to_vec(),
        compression_method: metadata.compression().clone(),
        samples,
    };

    let ts = tile.statistics().unwrap();

    ts
}

fn build_cell_array(
    arrays: Vec<ArrayRef>,
    metadata: Metadata,
) -> RaquetDataFusionResult<StructArray> {
    let binary_array = arrays[0]
        .as_any()
        .downcast_ref::<BinaryArray>()
        .expect("cast failed");

    let wkt_array = as_string_array(&arrays[1]);

    let mut out_builder = Float64Builder::new();

     for (binary, wkt) in multizip((binary_array, wkt_array)) {

       let (lon,lat) =wkt_to_lonlat(wkt.unwrap().to_string());
     }

   
    

    let values_fields = vec![
        Field::new("min", DataType::Float64, false),
        Field::new("max", DataType::Float64, false),
        Field::new("mean", DataType::Float64, false),
        Field::new("std_dev", DataType::Float64, false),
        Field::new("valid_count", DataType::UInt64, false),
    ];

    let fields = Fields::from(values_fields);

    let arrays: Vec<ArrayRef> = vec![
        Arc::new(min_builder.finish()),
        Arc::new(max_builder.finish()),
        Arc::new(mean_builder.finish()),
        Arc::new(std_dev_builder.finish()),
        Arc::new(valid_count_builder.finish()),
    ];
    let nulls = None;
    let arr = StructArray::new(fields, arrays, nulls);
    Ok(arr)
}
