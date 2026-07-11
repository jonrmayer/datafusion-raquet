use std::sync::{Arc, OnceLock};
use std::any::Any;

use crate::error::RaquetDataFusionResult;
use arrow::array::GenericListBuilder;

use arrow_array::{ArrayRef, BinaryArray, ListArray};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

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
            signature: Signature::exact(vec![DataType::Binary], Volatility::Immutable),
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
        "native_tile"
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

fn return_field_impl(args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    let metadata = Arc::new(Metadata::try_from(args.arg_fields[0].as_ref()).unwrap_or_default());

    let list_field = match metadata.data_type() {
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

fn build_cell_array(
    arrays: Vec<ArrayRef>,
    metadata: Metadata,
) -> RaquetDataFusionResult<ListArray> {
    let in_binary = arrays[0]
        .as_any()
        .downcast_ref::<BinaryArray>()
        .expect("cast failed");
    let ops: Operations = Operations::new(metadata.inner());
    let out_array = match metadata.data_type() {
        RasterDataType::Int8 => convert_list_array_i8(in_binary, ops)?.finish(),
        RasterDataType::UInt8 => convert_list_array_u8(in_binary, ops)?.finish(),
        RasterDataType::Int16 => convert_list_array_i16(in_binary, ops)?.finish(),
        RasterDataType::UInt16 => convert_list_array_u16(in_binary, ops)?.finish(),
        RasterDataType::Int32 => convert_list_array_i32(in_binary, ops)?.finish(),
        RasterDataType::UInt32 => convert_list_array_u32(in_binary, ops)?.finish(),
        RasterDataType::Int64 => convert_list_array_i64(in_binary, ops)?.finish(),
        RasterDataType::UInt64 => convert_list_array_u64(in_binary, ops)?.finish(),
        RasterDataType::Float32 => convert_list_array_f32(in_binary, ops)?.finish(),
        RasterDataType::Float64 => convert_list_array_f64(in_binary, ops)?.finish(),
        _ => todo!(),
    };

    Ok(out_array)
}

#[macro_export]
macro_rules! impl_convert_list_builder {
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
impl_convert_list_builder!(
    Int8Type,
    Int8Builder,
    decode_native_i8,
    convert_list_array_i8
);
impl_convert_list_builder!(
    UInt8Type,
    UInt8Builder,
    decode_native_u8,
    convert_list_array_u8
);

impl_convert_list_builder!(
    Int16Type,
    Int16Builder,
    decode_native_i16,
    convert_list_array_i16
);
impl_convert_list_builder!(
    UInt16Type,
    UInt16Builder,
    decode_native_u16,
    convert_list_array_u16
);

impl_convert_list_builder!(
    Int32Type,
    Int32Builder,
    decode_native_i32,
    convert_list_array_i32
);
impl_convert_list_builder!(
    UInt32Type,
    UInt32Builder,
    decode_native_u32,
    convert_list_array_u32
);

impl_convert_list_builder!(
    Int64Type,
    Int64Builder,
    decode_native_i64,
    convert_list_array_i64
);
impl_convert_list_builder!(
    UInt64Type,
    UInt64Builder,
    decode_native_u64,
    convert_list_array_u64
);

impl_convert_list_builder!(
    Float32Type,
    Float32Builder,
    decode_native_f32,
    convert_list_array_f32
);
impl_convert_list_builder!(
    Float64Type,
    Float64Builder,
    decode_native_f64,
    convert_list_array_f64
);

#[cfg(test)]
mod tests {

    use super::*;
    use crate::RaquetTable;
    use datafusion::prelude::{SessionConfig, SessionContext};

    #[tokio::test]
    async fn test_native_tile() {
        let path =
            "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet"
                .to_string();

        let ctx =
            SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

        ctx.register_udf(NativeTile::default().into());
        // ctx.register_udf(DecodeTile::default().into());
        let t = RaquetTable::from_path(path).await;

        let _ = ctx.register_table("solar", Arc::new(t));
        // let tci = ctx.table_provider("tci").await.unwrap();

        // let tci_schema = tci.schema();
        // println!("{:?}", tci_schema);

        let sql = "select native_tile(band_1) from solar where block<>0  ;";
        // let sql = "select count(*) from solar;";

        let df = ctx.sql(sql).await.unwrap();
        println!("{:?}", df.count().await);
        // df.show().await.unwrap();
    }
}
