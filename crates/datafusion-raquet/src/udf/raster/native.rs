use std::any::Any;
use std::sync::{Arc, OnceLock};

use crate::error::RaquetDataFusionResult;
use arrow::array::GenericListBuilder;

use arrow_array::{ArrayRef, BinaryArray, BinaryViewArray, LargeBinaryArray, ListArray};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    TypeSignature, Volatility,
};

use crate::udf::raster::utils::has_extension;

use rastertile_rs::Operations;
use rastertile_schema::Metadata;

use rastertile_rs::DataType as RasterDataType;

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

use arrow_array::builder::{ListBuilder, PrimitiveBuilder};

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct NativeTile {
    signature: Signature,
}

impl NativeTile {
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

impl Default for NativeTile {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for NativeTile {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn name(&self) -> &str {
        "raquet_band_native"
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
        let binary_field = &args.arg_fields[0];
        let binary_type = binary_field.data_type();
        let arrays = ColumnarValue::values_to_arrays(&args.args)?;

        match has_extension(binary_field.as_ref()) {
            true => {
                let column_metadata = Metadata::try_from(binary_field.as_ref()).unwrap_or_default();
                let struct_array = build_cell_array(arrays, binary_type, column_metadata)?;

                Ok(ColumnarValue::Array(Arc::new(struct_array)))
            }
            false => Err(DataFusionError::Internal("invoke_with_args".to_string())),
        }
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
    let binary_field = &args.arg_fields[0];

    match has_extension(binary_field.as_ref()) {
        true => {
            let column_metadata = Metadata::try_from(binary_field.as_ref()).unwrap_or_default();
            let list_field = match column_metadata.data_type() {
                RasterDataType::UInt8 => Some(Field::new_list_field(DataType::UInt8, true)),
                RasterDataType::Int8 => Some(Field::new_list_field(DataType::Int8, true)),
                RasterDataType::UInt16 => Some(Field::new_list_field(DataType::UInt16, true)),
                RasterDataType::Int16 => Some(Field::new_list_field(DataType::Int16, true)),
                RasterDataType::UInt32 => Some(Field::new_list_field(DataType::UInt32, true)),
                RasterDataType::Int32 => Some(Field::new_list_field(DataType::Int32, true)),
                RasterDataType::UInt64 => Some(Field::new_list_field(DataType::UInt64, true)),
                RasterDataType::Int64 => Some(Field::new_list_field(DataType::Int64, true)),
                RasterDataType::Float32 => Some(Field::new_list_field(DataType::Float32, true)),
                RasterDataType::Float64 => Some(Field::new_list_field(DataType::Float64, true)),
                _ => None,
            };

            let dt = DataType::List(Arc::new(list_field.unwrap()));
            let out_field: Field = Field::new("", dt, true);

            Ok(Arc::new(out_field))
        }
        false => Err(DataFusionError::Internal("return_field_impl".to_string())),
    }
}

fn build_cell_array(
    arrays: Vec<ArrayRef>,
    binary_type: &DataType,
    column_metadata: Metadata,
) -> RaquetDataFusionResult<ListArray> {

    let out_array = raquet_rs::raster::build_list_array(arrays, binary_type, column_metadata).expect("msg");
    // let out_array = match binary_type {
    //     DataType::Binary => {
    //         let in_binary = arrays[0]
    //             .as_any()
    //             .downcast_ref::<BinaryArray>()
    //             .expect("cast failed");
    //         let ops: Operations = Operations::new(column_metadata.inner());

    //         match column_metadata.data_type() {
    //             RasterDataType::Int8 => convert_binary_list_array_i8(in_binary, ops)?.finish(),
    //             RasterDataType::UInt8 => convert_binary_list_array_u8(in_binary, ops)?.finish(),
    //             RasterDataType::Int16 => convert_binary_list_array_i16(in_binary, ops)?.finish(),
    //             RasterDataType::UInt16 => convert_binary_list_array_u16(in_binary, ops)?.finish(),
    //             RasterDataType::Int32 => convert_binary_list_array_i32(in_binary, ops)?.finish(),
    //             RasterDataType::UInt32 => convert_binary_list_array_u32(in_binary, ops)?.finish(),
    //             RasterDataType::Int64 => convert_binary_list_array_i64(in_binary, ops)?.finish(),
    //             RasterDataType::UInt64 => convert_binary_list_array_u64(in_binary, ops)?.finish(),
    //             RasterDataType::Float32 => convert_binary_list_array_f32(in_binary, ops)?.finish(),
    //             RasterDataType::Float64 => convert_binary_list_array_f64(in_binary, ops)?.finish(),
    //             _ => todo!(),
    //         }
    //     }
    //     DataType::BinaryView => {
    //         let in_binary = arrays[0]
    //             .as_any()
    //             .downcast_ref::<BinaryViewArray>()
    //             .expect("cast failed");
    //         let ops: Operations = Operations::new(column_metadata.inner());

    //         match column_metadata.data_type() {
    //             RasterDataType::Int8 => convert_binaryview_list_array_i8(in_binary, ops)?.finish(),
    //             RasterDataType::UInt8 => convert_binaryview_list_array_u8(in_binary, ops)?.finish(),
    //             RasterDataType::Int16 => {
    //                 convert_binaryview_list_array_i16(in_binary, ops)?.finish()
    //             }
    //             RasterDataType::UInt16 => {
    //                 convert_binaryview_list_array_u16(in_binary, ops)?.finish()
    //             }
    //             RasterDataType::Int32 => {
    //                 convert_binaryview_list_array_i32(in_binary, ops)?.finish()
    //             }
    //             RasterDataType::UInt32 => {
    //                 convert_binaryview_list_array_u32(in_binary, ops)?.finish()
    //             }
    //             RasterDataType::Int64 => {
    //                 convert_binaryview_list_array_i64(in_binary, ops)?.finish()
    //             }
    //             RasterDataType::UInt64 => {
    //                 convert_binaryview_list_array_u64(in_binary, ops)?.finish()
    //             }
    //             RasterDataType::Float32 => {
    //                 convert_binaryview_list_array_f32(in_binary, ops)?.finish()
    //             }
    //             RasterDataType::Float64 => {
    //                 convert_binaryview_list_array_f64(in_binary, ops)?.finish()
    //             }
    //             _ => todo!(),
    //         }
    //     }
    //     DataType::LargeBinary => {
    //         let in_binary = arrays[0]
    //             .as_any()
    //             .downcast_ref::<LargeBinaryArray>()
    //             .expect("cast failed");
    //         let ops: Operations = Operations::new(column_metadata.inner());

    //         match column_metadata.data_type() {
    //             RasterDataType::Int8 => convert_largebinary_list_array_i8(in_binary, ops)?.finish(),
    //             RasterDataType::UInt8 => {
    //                 convert_largebinary_list_array_u8(in_binary, ops)?.finish()
    //             }
    //             RasterDataType::Int16 => {
    //                 convert_largebinary_list_array_i16(in_binary, ops)?.finish()
    //             }
    //             RasterDataType::UInt16 => {
    //                 convert_largebinary_list_array_u16(in_binary, ops)?.finish()
    //             }
    //             RasterDataType::Int32 => {
    //                 convert_largebinary_list_array_i32(in_binary, ops)?.finish()
    //             }
    //             RasterDataType::UInt32 => {
    //                 convert_largebinary_list_array_u32(in_binary, ops)?.finish()
    //             }
    //             RasterDataType::Int64 => {
    //                 convert_largebinary_list_array_i64(in_binary, ops)?.finish()
    //             }
    //             RasterDataType::UInt64 => {
    //                 convert_largebinary_list_array_u64(in_binary, ops)?.finish()
    //             }
    //             RasterDataType::Float32 => {
    //                 convert_largebinary_list_array_f32(in_binary, ops)?.finish()
    //             }
    //             RasterDataType::Float64 => {
    //                 convert_largebinary_list_array_f64(in_binary, ops)?.finish()
    //             }
    //             _ => todo!(),
    //         }
    //     }

    //     _ => unreachable!(),
    // };

    Ok(out_array)
}

// use arrow_array::types::ArrowPrimitiveType;
// /// A single generic function replacing all 30 macro-generated variants.
// pub fn convert_to_list_array<'a, A, T, F>(in_binary: A, ops: &Operations, decode_fn: F) -> ListArray
// where
//     A: IntoIterator<Item = Option<&'a [u8]>>,
//     T: ArrowPrimitiveType,
//     // Note that the closure bound now expects an inner vector of Option elements
//     F: Fn(&Operations, Option<&[u8]>) -> RaquetDataFusionResult<Option<Vec<Option<T::Native>>>>,
// {
//     let values_builder = PrimitiveBuilder::<T>::new();
//     let mut builder = ListBuilder::new(values_builder);

//     for input in in_binary {
//         // 1. Unpack the outer Option block safely using ?

//         //  RaquetDataFusionResult<Option<Vec<i8>>>
//                 // let output_option: Option<Vec<<T::Native>>> = decode_fn(ops, input).expect("....");
//         let output_option: Option<Vec<Option<T::Native>>> = decode_fn(ops, input).expect("....");

//         match output_option {
//             Some(vector) => {
//                 // // 2. Loop through individual pixels and append them one by one
//                 // for pixel in vector {
//                 //     builder.values().append_option(pixel);
//                 // }
//                 // ALTERNATIVE: Since primitive builders implement Extend, you can also use:
//                 builder.values().extend(vector);

//                 // 3. Mark the row index slot as valid
//                 builder.append(true);
//             }
//             None => {
//                 // 4. Mark the whole row slot as a database NULL list index entry
//                 builder.append(false);
//             }
//         }
//     }

//     builder.finish()
// }

// use crate::error::RaquetDataFusionError;
// fn build_cell_array(
//     arrays: Vec<ArrayRef>,
//     binary_type: &DataType,
//     column_metadata: Metadata,
// ) -> RaquetDataFusionResult<ListArray> {
//     let ops = Operations::new(column_metadata.inner());
//     let target_array = &arrays[0];

//     // Local dispatcher macro mapping metadata to our clean, unified helper function
//     macro_rules! dispatch_types {
//         ($input_array:expr) => {
//             match column_metadata.data_type() {
//                 RasterDataType::Int8 => {
//                     convert_to_list_array::<_, Int8Type, _>($input_array.iter(), &ops, decode_i8)
//                 }
//                 // RasterDataType::UInt8 => {
//                 //     convert_to_list_array::<_, UInt8Type, _>($input_array.iter(), &ops, decode_u8)
//                 // }
//                 // RasterDataType::Int16 => {
//                 //     convert_to_list_array::<_, Int16Type, _>($input_array.iter(), &ops, decode_i16)
//                 // }
//                 // RasterDataType::UInt16 => convert_to_list_array::<_, UInt16Type, _>(
//                 //     $input_array.iter(),
//                 //     &ops,
//                 //     decode_u16,
//                 // ),
//                 // RasterDataType::Int32 => {
//                 //     convert_to_list_array::<_, Int32Type, _>($input_array.iter(), &ops, decode_i32)
//                 // }
//                 // RasterDataType::UInt32 => convert_to_list_array::<_, UInt32Type, _>(
//                 //     $input_array.iter(),
//                 //     &ops,
//                 //     decode_u32,
//                 // ),
//                 // RasterDataType::Int64 => {
//                 //     convert_to_list_array::<_, Int64Type, _>($input_array.iter(), &ops, decode_i64)
//                 // }
//                 // RasterDataType::UInt64 => convert_to_list_array::<_, UInt64Type, _>(
//                 //     $input_array.iter(),
//                 //     &ops,
//                 //     decode_u64,
//                 // ),
//                 // RasterDataType::Float32 => convert_to_list_array::<_, Float32Type, _>(
//                 //     $input_array.iter(),
//                 //     &ops,
//                 //     decode_f32,
//                 // ),
//                 // RasterDataType::Float64 => convert_to_list_array::<_, Float64Type, _>(
//                 //     $input_array.iter(),
//                 //     &ops,
//                 //     decode_f64,
//                 // ),
//                 _ => {
//                     return Err(DataFusionError::Internal(
//                         "Unsupported raster data type layout".to_string(),
//                     )
//                     .into())
//                 }
//             }
//         };
//     }

//     let out_array = match binary_type {
//         DataType::Binary => {
//             let casted = target_array
//                 .as_any()
//                 .downcast_ref::<BinaryArray>()
//                 .expect("cast failed");
//             dispatch_types!(casted)
//         }
//         DataType::BinaryView => {
//             let casted = target_array
//                 .as_any()
//                 .downcast_ref::<BinaryViewArray>()
//                 .expect("cast failed");
//             dispatch_types!(casted)
//         }
//         DataType::LargeBinary => {
//             let casted = target_array
//                 .as_any()
//                 .downcast_ref::<LargeBinaryArray>()
//                 .expect("cast failed");
//             dispatch_types!(casted)
//         }
//         _ => unreachable!(),
//     };

//     Ok(out_array)
// }

// let output_option: Option<Vec<Option<T::Native>>
// Vec<Option<i8>>

// fn decode_i8(o: &Operations, idx: Option<&[u8]>) -> RaquetDataFusionResult<Vec<Option<i8>>> {
//     o.decode_native_i8(idx)
// }
// fn decode_u8(o: &Operations, idx: Option<&[u8]>) -> RaquetDataFusionResult<Option<Vec<u8>>> {
//     o.decode_native_u8(idx)
// }
// fn decode_i16(o: &Operations, idx: Option<&[u8]>) -> RaquetDataFusionResult<Option<Vec<i16>>> {
//     o.decode_native_i16(idx)
// }
// fn decode_u16(o: &Operations, idx: Option<&[u8]>) -> RaquetDataFusionResult<Option<Vec<u16>>> {
//     o.decode_native_u16(idx)
// }
// fn decode_i32(o: &Operations, idx: Option<&[u8]>) -> RaquetDataFusionResult<Option<Vec<i32>>> {
//     o.decode_native_i32(idx)
// }
// fn decode_u32(o: &Operations, idx: Option<&[u8]>) -> RaquetDataFusionResult<Option<Vec<u32>>> {
//     o.decode_native_u32(idx)
// }
// fn decode_i64(o: &Operations, idx: Option<&[u8]>) -> RaquetDataFusionResult<Option<Vec<i64>>> {
//     o.decode_native_i64(idx)
// }
// fn decode_u64(o: &Operations, idx: Option<&[u8]>) -> RaquetDataFusionResult<Option<Vec<u64>>> {
//     o.decode_native_u64(idx)
// }
// fn decode_f32(o: &Operations, idx: Option<&[u8]>) -> RaquetDataFusionResult<Option<Vec<f32>>> {
//     o.decode_native_f32(idx)
// }
// fn decode_f64(o: &Operations, idx: Option<&[u8]>) -> RaquetDataFusionResult<Option<Vec<f64>>> {
//     o.decode_native_f64(idx)
// }

#[macro_export]
macro_rules! impl_binary_convert_list_builder {
    ($list_type:ident, $builder:ident,$decode:ident, $name:ident) => {
        pub fn $name(
            in_binary: &arrow_array::GenericByteArray<arrow::datatypes::GenericBinaryType<i32>>,
            ops: Operations,
        ) -> RaquetDataFusionResult<GenericListBuilder<i32, PrimitiveBuilder<$list_type>>> {
            let values_builder = $builder::new();
            let mut builder = ListBuilder::new(values_builder);

            for input in in_binary.iter() {
                let output = ops.$decode(input)?;
                builder.append_value(output);
            }

            Ok(builder)
        }
    };
}

#[macro_export]
macro_rules! impl_binaryview_convert_list_builder {
    ($list_type:ident, $builder:ident,$decode:ident, $name:ident) => {
        pub fn $name(
            in_binary: &arrow_array::GenericByteViewArray<arrow::datatypes::BinaryViewType>,
            ops: Operations,
        ) -> RaquetDataFusionResult<GenericListBuilder<i32, PrimitiveBuilder<$list_type>>> {
            let values_builder = $builder::new();
            let mut builder = ListBuilder::new(values_builder);

            for input in in_binary.iter() {
                let output = ops.$decode(input)?;
                builder.append_value(output);
            }

            Ok(builder)
        }
    };
}

#[macro_export]
macro_rules! impl_largebinary_convert_list_builder {
    ($list_type:ident, $builder:ident,$decode:ident, $name:ident) => {
        pub fn $name(
            in_binary: &arrow_array::GenericByteArray<arrow::datatypes::GenericBinaryType<i64>>,
            ops: Operations,
        ) -> RaquetDataFusionResult<GenericListBuilder<i32, PrimitiveBuilder<$list_type>>> {
            let values_builder = $builder::new();
            let mut builder = ListBuilder::new(values_builder);

            for input in in_binary.iter() {
                let output = ops.$decode(input)?;
                builder.append_value(output);
            }

            Ok(builder)
        }
    };
}

// Int8Type
impl_binary_convert_list_builder!(
    Int8Type,
    Int8Builder,
    decode_native_i8,
    convert_binary_list_array_i8
);
impl_largebinary_convert_list_builder!(
    Int8Type,
    Int8Builder,
    decode_native_i8,
    convert_largebinary_list_array_i8
);

impl_binaryview_convert_list_builder!(
    Int8Type,
    Int8Builder,
    decode_native_i8,
    convert_binaryview_list_array_i8
);
// UInt8Type

impl_binary_convert_list_builder!(
    UInt8Type,
    UInt8Builder,
    decode_native_u8,
    convert_binary_list_array_u8
);

impl_largebinary_convert_list_builder!(
    UInt8Type,
    UInt8Builder,
    decode_native_u8,
    convert_largebinary_list_array_u8
);
impl_binaryview_convert_list_builder!(
    UInt8Type,
    UInt8Builder,
    decode_native_u8,
    convert_binaryview_list_array_u8
);

// Int16Type
impl_binary_convert_list_builder!(
    Int16Type,
    Int16Builder,
    decode_native_i16,
    convert_binary_list_array_i16
);

impl_largebinary_convert_list_builder!(
    Int16Type,
    Int16Builder,
    decode_native_i16,
    convert_largebinary_list_array_i16
);

impl_binaryview_convert_list_builder!(
    Int16Type,
    Int16Builder,
    decode_native_i16,
    convert_binaryview_list_array_i16
);

// UInt16Type

impl_binary_convert_list_builder!(
    UInt16Type,
    UInt16Builder,
    decode_native_u16,
    convert_binary_list_array_u16
);

impl_largebinary_convert_list_builder!(
    UInt16Type,
    UInt16Builder,
    decode_native_u16,
    convert_largebinary_list_array_u16
);

impl_binaryview_convert_list_builder!(
    UInt16Type,
    UInt16Builder,
    decode_native_u16,
    convert_binaryview_list_array_u16
);

// Int32Type
impl_binary_convert_list_builder!(
    Int32Type,
    Int32Builder,
    decode_native_i32,
    convert_binary_list_array_i32
);

impl_largebinary_convert_list_builder!(
    Int32Type,
    Int32Builder,
    decode_native_i32,
    convert_largebinary_list_array_i32
);

impl_binaryview_convert_list_builder!(
    Int32Type,
    Int32Builder,
    decode_native_i32,
    convert_binaryview_list_array_i32
);

// UInt32Type
impl_binary_convert_list_builder!(
    UInt32Type,
    UInt32Builder,
    decode_native_u32,
    convert_binary_list_array_u32
);

impl_largebinary_convert_list_builder!(
    UInt32Type,
    UInt32Builder,
    decode_native_u32,
    convert_largebinary_list_array_u32
);

impl_binaryview_convert_list_builder!(
    UInt32Type,
    UInt32Builder,
    decode_native_u32,
    convert_binaryview_list_array_u32
);

// Int64Type

impl_binary_convert_list_builder!(
    Int64Type,
    Int64Builder,
    decode_native_i64,
    convert_binary_list_array_i64
);

impl_largebinary_convert_list_builder!(
    Int64Type,
    Int64Builder,
    decode_native_i64,
    convert_largebinary_list_array_i64
);

impl_binaryview_convert_list_builder!(
    Int64Type,
    Int64Builder,
    decode_native_i64,
    convert_binaryview_list_array_i64
);

// UInt64Type
impl_binary_convert_list_builder!(
    UInt64Type,
    UInt64Builder,
    decode_native_u64,
    convert_binary_list_array_u64
);

impl_largebinary_convert_list_builder!(
    UInt64Type,
    UInt64Builder,
    decode_native_u64,
    convert_largebinary_list_array_u64
);

impl_binaryview_convert_list_builder!(
    UInt64Type,
    UInt64Builder,
    decode_native_u64,
    convert_binaryview_list_array_u64
);

// Float32Type

impl_binary_convert_list_builder!(
    Float32Type,
    Float32Builder,
    decode_native_f32,
    convert_binary_list_array_f32
);

impl_largebinary_convert_list_builder!(
    Float32Type,
    Float32Builder,
    decode_native_f32,
    convert_largebinary_list_array_f32
);

impl_binaryview_convert_list_builder!(
    Float32Type,
    Float32Builder,
    decode_native_f32,
    convert_binaryview_list_array_f32
);

// Float64Type
impl_binary_convert_list_builder!(
    Float64Type,
    Float64Builder,
    decode_native_f64,
    convert_binary_list_array_f64
);

impl_largebinary_convert_list_builder!(
    Float64Type,
    Float64Builder,
    decode_native_f64,
    convert_largebinary_list_array_f64
);

impl_binaryview_convert_list_builder!(
    Float64Type,
    Float64Builder,
    decode_native_f64,
    convert_binaryview_list_array_f64
);
