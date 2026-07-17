use crate::error::RaquetDataFusionResult;
use arrow::datatypes::Fields;
use arrow_array::builder::{Float64Builder, UInt64Builder};
use std::any::Any;
use std::sync::{Arc, OnceLock};

use arrow_array::{ArrayRef, BinaryArray, BinaryViewArray, LargeBinaryArray, StructArray};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    TypeSignature, Volatility,
};

use rastertile_schema::Metadata;

use rastertile_rs::Operations;

use crate::udf::raster::utils::has_extension;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct StatisticsTile {
    signature: Signature,
}

impl StatisticsTile {
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

    fn data_type(&self) -> DataType {
        let values_fields = vec![
            Field::new("min", DataType::Float64, false),
            Field::new("max", DataType::Float64, false),
            Field::new("mean", DataType::Float64, false),
            Field::new("std_dev", DataType::Float64, false),
            Field::new("valid_count", DataType::UInt64, false),
        ];
        DataType::Struct(values_fields.into())
    }
    fn to_field<N: Into<String>>(&self, name: N, nullable: bool) -> Field {
        Field::new(name, self.data_type(), nullable)
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
        "raquet_band_statistics"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        Err(DataFusionError::Internal("return_type".to_string()))
    }

    fn return_field_from_args(&self, _args: ReturnFieldArgs) -> Result<FieldRef> {
        Ok(Arc::new(self.to_field("", false)))
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
                "Return a statistics struct from a band.",
                "raquet_band_statistics(band)",
            )
            .with_argument("band", "band value")
            .build()
        }))
    }
}

fn build_cell_array(
    arrays: Vec<ArrayRef>,
    binary_type: &DataType,
    column_metadata: Metadata,
) -> RaquetDataFusionResult<StructArray> {
    let mut min_builder = Float64Builder::new();
    let mut max_builder = Float64Builder::new();
    let mut mean_builder = Float64Builder::new();
    let mut std_dev_builder = Float64Builder::new();
    let mut valid_count_builder = UInt64Builder::new();

    match binary_type {
        DataType::Binary => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryArray>()
                .expect("cast failed");
            let ops: Operations = Operations::new(column_metadata.inner());
            for input in in_binary.iter() {
                let output = ops.statistics(input)?;
                min_builder.append_value(output.min);
                max_builder.append_value(output.max);
                mean_builder.append_value(output.mean);
                std_dev_builder.append_value(output.std_dev);
                valid_count_builder.append_value(output.valid_count);
            }
        }
        DataType::BinaryView => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<BinaryViewArray>()
                .expect("cast failed");
            let ops: Operations = Operations::new(column_metadata.inner());
            for input in in_binary.iter() {
                let output = ops.statistics(input)?;
                min_builder.append_value(output.min);
                max_builder.append_value(output.max);
                mean_builder.append_value(output.mean);
                std_dev_builder.append_value(output.std_dev);
                valid_count_builder.append_value(output.valid_count);
            }
        }
        DataType::LargeBinary => {
            let in_binary = arrays[0]
                .as_any()
                .downcast_ref::<LargeBinaryArray>()
                .expect("cast failed");
            let ops: Operations = Operations::new(column_metadata.inner());
            for input in in_binary.iter() {
                let output = ops.statistics(input)?;
                min_builder.append_value(output.min);
                max_builder.append_value(output.max);
                mean_builder.append_value(output.mean);
                std_dev_builder.append_value(output.std_dev);
                valid_count_builder.append_value(output.valid_count);
            }
        }

        _ => unreachable!(),
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
