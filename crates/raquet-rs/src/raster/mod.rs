use arrow::array::GenericListBuilder;
use arrow_array::types::ArrowPrimitiveType;

use arrow_array::{ArrayRef, BinaryArray, BinaryViewArray, LargeBinaryArray, ListArray};
use arrow_schema::{DataType, Field, FieldRef};

use arrow_array::builder::{ListBuilder, PrimitiveBuilder};
use rastertile_schema::Metadata;

use rastertile_rs::DataType as RasterDataType;

use rastertile_rs::Operations;
use rastertile_rs::{OperationsError, OperationsResult};

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

/// A single generic function replacing all 30 macro-generated variants.
pub fn convert_to_list_array<'a, A, T, F>(in_binary: A, ops: &Operations, ops_fn: F) -> ListArray
where
    A: IntoIterator<Item = Option<&'a [u8]>>,
    T: ArrowPrimitiveType,
    // Note that the closure bound now expects an inner vector of Option elements
    F: Fn(&Operations, Option<&[u8]>) -> OperationsResult<Vec<Option<T::Native>>>,
{
    let values_builder = PrimitiveBuilder::<T>::new();
    let mut builder = ListBuilder::new(values_builder);

    for input in in_binary {
        let output: Vec<Option<T::Native>> = ops_fn(ops, input).expect("....");
        builder.append_value(output);
        // builder.values().append(output_option);
    }

    builder.finish()
}

pub fn build_list_array(
    arrays: Vec<ArrayRef>,
    binary_type: &DataType,
    column_metadata: Metadata,
) -> OperationsResult<ListArray> {
    let ops = Operations::new(column_metadata.inner());
    let target_array = &arrays[0];

    // Local dispatcher macro mapping metadata to our clean, unified helper function
    macro_rules! dispatch_types {
        ($input_array:expr) => {
            match column_metadata.data_type() {
                RasterDataType::Int8 => {
                    convert_to_list_array::<_, Int8Type, _>($input_array.iter(), &ops, decode_i8)
                }
                RasterDataType::UInt8 => {
                    convert_to_list_array::<_, UInt8Type, _>($input_array.iter(), &ops, decode_u8)
                }
                RasterDataType::Int16 => {
                    convert_to_list_array::<_, Int16Type, _>($input_array.iter(), &ops, decode_i16)
                }
                RasterDataType::UInt16 => convert_to_list_array::<_, UInt16Type, _>(
                    $input_array.iter(),
                    &ops,
                    decode_u16,
                ),
                RasterDataType::Int32 => {
                    convert_to_list_array::<_, Int32Type, _>($input_array.iter(), &ops, decode_i32)
                }
                RasterDataType::UInt32 => convert_to_list_array::<_, UInt32Type, _>(
                    $input_array.iter(),
                    &ops,
                    decode_u32,
                ),
                RasterDataType::Int64 => {
                    convert_to_list_array::<_, Int64Type, _>($input_array.iter(), &ops, decode_i64)
                }
                RasterDataType::UInt64 => convert_to_list_array::<_, UInt64Type, _>(
                    $input_array.iter(),
                    &ops,
                    decode_u64,
                ),
                RasterDataType::Float32 => convert_to_list_array::<_, Float32Type, _>(
                    $input_array.iter(),
                    &ops,
                    decode_f32,
                ),
                RasterDataType::Float64 => convert_to_list_array::<_, Float64Type, _>(
                    $input_array.iter(),
                    &ops,
                    decode_f64,
                ),
                _ => {
                    return Err(OperationsError::General(
                        "Unsupported raster data type layout".to_string(),
                    )
                    .into())
                }
            }
        };
    }

    let out_array = match binary_type {
        DataType::Binary => {
            let casted = target_array
                .as_any()
                .downcast_ref::<BinaryArray>()
                .expect("cast failed");
            dispatch_types!(casted)
        }
        DataType::BinaryView => {
            let casted = target_array
                .as_any()
                .downcast_ref::<BinaryViewArray>()
                .expect("cast failed");
            dispatch_types!(casted)
        }
        DataType::LargeBinary => {
            let casted = target_array
                .as_any()
                .downcast_ref::<LargeBinaryArray>()
                .expect("cast failed");
            dispatch_types!(casted)
        }
        _ => unreachable!(),
    };

    Ok(out_array)
}

fn decode_i8(o: &Operations, idx: Option<&[u8]>) -> OperationsResult<Vec<Option<i8>>> {
    o.decode_native_i8(idx)
}

fn decode_u8(o: &Operations, idx: Option<&[u8]>) -> OperationsResult<Vec<Option<u8>>> {
    o.decode_native_u8(idx)
}
fn decode_i16(o: &Operations, idx: Option<&[u8]>) -> OperationsResult<Vec<Option<i16>>> {
    o.decode_native_i16(idx)
}
fn decode_u16(o: &Operations, idx: Option<&[u8]>) -> OperationsResult<Vec<Option<u16>>> {
    o.decode_native_u16(idx)
}
fn decode_i32(o: &Operations, idx: Option<&[u8]>) -> OperationsResult<Vec<Option<i32>>> {
    o.decode_native_i32(idx)
}
fn decode_u32(o: &Operations, idx: Option<&[u8]>) -> OperationsResult<Vec<Option<u32>>> {
    o.decode_native_u32(idx)
}
fn decode_i64(o: &Operations, idx: Option<&[u8]>) -> OperationsResult<Vec<Option<i64>>> {
    o.decode_native_i64(idx)
}
fn decode_u64(o: &Operations, idx: Option<&[u8]>) -> OperationsResult<Vec<Option<u64>>> {
    o.decode_native_u64(idx)
}
fn decode_f32(o: &Operations, idx: Option<&[u8]>) -> OperationsResult<Vec<Option<f32>>> {
    o.decode_native_f32(idx)
}
fn decode_f64(o: &Operations, idx: Option<&[u8]>) -> OperationsResult<Vec<Option<f64>>> {
    o.decode_native_f64(idx)
}
