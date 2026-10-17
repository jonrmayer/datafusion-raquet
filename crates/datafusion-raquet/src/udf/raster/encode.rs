use arrow::array::{Array, ArrayRef, AsArray, BinaryArray, ListArray, make_array};
use arrow::datatypes::{DataType as ArrowDataType, Field};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
// use datafusion_common::{exec_err, Result, ScalarValue};
// use datafusion_expr::{ColumnarValue, ScalarUDFImpl, Signature, Volatility, UdfArgs, Expr};
// use datafusion_expr::return_field::ReturnField;

use arrow_schema::{DataType, FieldRef};

use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    TypeSignature, Volatility,
};

use crate::error::RaquetDataFusionResult;

use datafusion::error::{DataFusionError, Result};

use datafusion::common::ScalarValue;

use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;

use arrow_array::builder::{BinaryViewBuilder, GenericBinaryBuilder};

use rastertile_rs::DataType as RasterDataType;

// // Real structures brought in from crates/rastertile-schema
// use rastertile_schema::metadata::BandMetadata;
// use rastertile::DataType as RasterDataType;
// use rastertile::operations::Operations;
// use rastertile::metadata::Metadata as RuntimeMetadata;

use rastertile_schema::Metadata;

use rastertile_rs::Operations;

use rastertile_schema::{RasterArrowType, RasterType};

use rastertile_rs::Metadata as RasterMetadata;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct EncodeTile {
    signature: Signature,
}

impl EncodeTile {
    pub fn new() -> Self {
        let numeric_types = vec![
            DataType::Int8,
            DataType::UInt8,
            DataType::Int16,
            DataType::UInt16,
            DataType::Int32,
            DataType::UInt32,
            DataType::Int64,
            DataType::UInt64,
            DataType::Float32,
            DataType::Float64,
        ];

        let mut allowed_signatures = Vec::new();

        for num_type in numeric_types {
            let list_field = Arc::new(Field::new("item", num_type, true));

            allowed_signatures.push(TypeSignature::Exact(vec![
                DataType::List(list_field),
                DataType::Utf8, // tile_size
                DataType::Utf8, // binary_type
                DataType::Utf8, // data_type
                DataType::Utf8, // no_data
                DataType::Utf8, // compression
            ]));
        }
        Self {
            signature: Signature::one_of(allowed_signatures, Volatility::Immutable),
        }
    }
}

impl Default for EncodeTile {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for EncodeTile {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn name(&self) -> &str {
        "raquet_band_encode"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        Err(DataFusionError::Internal("return_type".to_string()))
    }

    fn return_field_from_args(&self, args: ReturnFieldArgs) -> Result<FieldRef> {
        return_field_impl(args)
    }

    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let return_field = args.return_field;
        let arrays = ColumnarValue::values_to_arrays(&args.args)?;
        let typ = return_field.extension_type::<RasterType>();
        let column_metadata = typ.metadata().as_ref().clone();
        let col_value = build_cell_array(arrays, column_metadata)?;
        Ok(col_value)
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

fn return_field_impl(args: ReturnFieldArgs) -> Result<FieldRef> {
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

    let raster_type = RasterType::new(metadata);
    let raster_arrow_type = RasterArrowType::Raster(raster_type);
    Ok(Arc::new(raster_arrow_type.to_field("", true)))
}

fn build_cell_array(
    arrays: Vec<ArrayRef>,
    column_metadata: Metadata,
) -> RaquetDataFusionResult<ColumnarValue> {
    // let list_array = arrays[0]
    let list_array: &ListArray = arrays[0]
        .as_ref()
        .as_any()
        .downcast_ref::<ListArray>()
        .expect("Expected a ListArray");

    // let list_array = arrays[0]
    //             .as_any()
    //             .downcast_ref::<ListArray>()
    //             .expect("cast failed");
    // let mut builder = GenericBinaryBuilder::<i32>::new();

    let ops: Operations = Operations::new(column_metadata.inner());

    let mut builder = match column_metadata.data_type() {
        // Signed Integers
        RasterDataType::Int8 => convert_list_array_i8_binary_builder(list_array, ops)?,
        RasterDataType::Int16 => convert_list_array_i16_binary_builder(list_array, ops)?,
        RasterDataType::Int32 => convert_list_array_i32_binary_builder(list_array, ops)?,
        RasterDataType::Int64 => convert_list_array_i64_binary_builder(list_array, ops)?,

        // Unsigned Integers
        RasterDataType::UInt8 => convert_list_array_u8_binary_builder(list_array, ops)?,
        RasterDataType::UInt16 => convert_list_array_u16_binary_builder(list_array, ops)?,
        RasterDataType::UInt32 => convert_list_array_u32_binary_builder(list_array, ops)?,
        RasterDataType::UInt64 => convert_list_array_u64_binary_builder(list_array, ops)?,

        // Floating-Point Numbers
        RasterDataType::Float32 => convert_list_array_f32_binary_builder(list_array, ops)?,
        RasterDataType::Float64 => convert_list_array_f64_binary_builder(list_array, ops)?,
        _ => todo!(),
    };
    // }

    let point_arr = builder.finish();
    Ok(ColumnarValue::Array(Arc::new(point_arr)))
}

use arrow_array::builder::{Int8Builder, UInt8Builder};
use arrow_array::types::{Int8Type, UInt8Type};

use arrow_array::builder::{Int16Builder, UInt16Builder};
use arrow_array::types::{Int16Type, UInt16Type};

use arrow_array::builder::{Int32Builder, UInt32Builder};
use arrow_array::types::{Int32Type, UInt32Type};

use arrow_array::builder::{Int64Builder, UInt64Builder};
use arrow_array::types::{Int64Type, UInt64Type};

use arrow_array::builder::{Float32Builder, Float64Builder};
use arrow_array::types::{Float32Type, Float64Type};

use crate::error::RaquetDataFusionError;

// pub fn convert_list_array_i8_binary_builder(
//     in_list: &arrow_array::GenericListArray<i32>,
//     ops: Operations,
// ) -> RaquetDataFusionResult<GenericBinaryBuilder<i32>> {
//     let mut builder = GenericBinaryBuilder::<i32>::new();
//     for opt_input in in_list.iter() {
//         match opt_input {
//             Some(input) => {
//                 // 1. Correctly target Int8Type
//                 let primitive = input.as_primitive::<arrow_array::types::Int8Type>();

//                 // 2. Clone the raw internal i8 slice
//                 let values_vec: Vec<i8> = primitive.values().to_vec();

//                 // 3. Process the vec and map your custom OperationsError into RaquetDataFusionError
//                 let output = ops
//                     .encode_native_i8(values_vec)
//                     .map_err(|e| RaquetDataFusionError::from(e))?; // Adjust mapping to your actual Error enum names

//                 builder.append_value(output);
//             }
//             None => {
//                 // 4. Safely handle internal Arrow null list slots
//                 builder.append_null();
//             }
//         }
//     }

//     Ok(builder)
// }

#[macro_export]
macro_rules! impl_list_convert_binary_builder {
    ($array_type:ident, $internal_type:ident,$encode:ident, $name:ident) => {
        pub fn $name(
            in_list: &arrow_array::GenericListArray<i32>,
            ops: Operations,
        ) -> RaquetDataFusionResult<GenericBinaryBuilder<i32>> {
            let mut builder = GenericBinaryBuilder::<i32>::new();
            for opt_input in in_list.iter() {
                match opt_input {
                    Some(input) => {
                        // 1. Correctly target Int8Type
                        let primitive = input.as_primitive::<arrow_array::types::$array_type>();

                        // 2. Clone the raw internal i8 slice
                        let values_vec: Vec<$internal_type> = primitive.values().to_vec();

                        // 3. Process the vec and map your custom OperationsError into RaquetDataFusionError
                        let output = ops
                            .$encode(values_vec)
                            .map_err(|e| RaquetDataFusionError::from(e))?; // Adjust mapping to your actual Error enum names

                        builder.append_value(output);
                    }
                    None => {
                        // 4. Safely handle internal Arrow null list slots
                        builder.append_null();
                    }
                }
            }

            Ok(builder)
        }
    };
}

// Int8Type
impl_list_convert_binary_builder!(
    Int8Type,
    i8,
    encode_native_i8,
    convert_list_array_i8_binary_builder
);

// Int16Type
impl_list_convert_binary_builder!(
    Int16Type,
    i16,
    encode_native_i16,
    convert_list_array_i16_binary_builder
);

// Int32Type
impl_list_convert_binary_builder!(
    Int32Type,
    i32,
    encode_native_i32,
    convert_list_array_i32_binary_builder
);

// Int64Type
impl_list_convert_binary_builder!(
    Int64Type,
    i64,
    encode_native_i64,
    convert_list_array_i64_binary_builder
);

// UInt8Type
impl_list_convert_binary_builder!(
    UInt8Type,
    u8,
    encode_native_u8,
    convert_list_array_u8_binary_builder
);

// UInt16Type
impl_list_convert_binary_builder!(
    UInt16Type,
    u16,
    encode_native_u16,
    convert_list_array_u16_binary_builder
);

// UInt32Type
impl_list_convert_binary_builder!(
    UInt32Type,
    u32,
    encode_native_u32,
    convert_list_array_u32_binary_builder
);

// UInt64Type
impl_list_convert_binary_builder!(
    UInt64Type,
    u64,
    encode_native_u64,
    convert_list_array_u64_binary_builder
);

// Float32Type
impl_list_convert_binary_builder!(
    Float32Type,
    f32,
    encode_native_f32,
    convert_list_array_f32_binary_builder
);

// Float64Type
impl_list_convert_binary_builder!(
    Float64Type,
    f64,
    encode_native_f64,
    convert_list_array_f64_binary_builder
);
