use std::any::Any;
use std::sync::{Arc, OnceLock};

use crate::error::RaquetDataFusionResult;
use arrow::datatypes::{Fields, Int8Type};
use arrow_array::builder::{Float64Builder, UInt64Builder};
use arrow_array::cast::{AsArray, as_primitive_array, as_string_array};
use arrow_array::types::Int64Type;
use arrow_array::{ArrayRef, BinaryArray, Float64Array, PrimitiveArray, StructArray};
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

use rastertile_rs::{CompressionFormat, Operations};

use quadbin_geo_rs::wkt_to_lonlat;

// use crate::udf::raster::utils::{convert_f32,get_tile,get_pixel};

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct RaquetPixel {
    signature: Signature,
}

impl RaquetPixel {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(
                vec![DataType::Binary, DataType::Int64, DataType::Int64],
                Volatility::Immutable,
            ),
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

impl Default for RaquetPixel {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for RaquetPixel {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "raquet_pixel"
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
                "raquet_pixel(band,pixel_x,pixel_y)",
            )
            .with_argument("band", "band value")
            .with_argument("pixel_x", "pixel_x value")
            .with_argument("pixel_y", "pixel_y value")
            .build()
        }))
    }
}

fn build_cell_array(
    arrays: Vec<ArrayRef>,
    metadata: Metadata,
) -> RaquetDataFusionResult<Float64Array> {
    let binary_array = arrays[0]
        .as_any()
        .downcast_ref::<BinaryArray>()
        .expect("cast failed");

    let pixel_x_array = arrays[1].as_primitive::<Int64Type>();
    let pixel_y_array = arrays[2].as_primitive::<Int64Type>();

    let mut out_builder = Float64Builder::new();
    let ops: Operations = Operations::new(metadata.inner());
    for (binary, pixel_x, pixel_y) in multizip((binary_array, pixel_x_array, pixel_y_array)) {
        let value = ops.getpixel(binary, pixel_x.unwrap() as u64, pixel_y.unwrap() as u64)?;
        out_builder.append_value(value);
    }

    let point_arr = out_builder.finish();

    Ok(point_arr)
}
#[cfg(test)]
mod tests {
    // use datafusion::prelude::SessionContext;

    use super::*;
    use crate::RaquetTable;
    use crate::udf::quadbin::QuadBinToPixelXY;
    use datafusion::prelude::{SessionConfig, SessionContext};

    #[tokio::test]
    async fn test_raquet_pixel() {
        let path =
            "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet"
                .to_string();

        let ctx =
            SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

        ctx.register_udf(RaquetPixel::default().into());
        ctx.register_udf(QuadBinToPixelXY::default().into());
        let t = RaquetTable::from_path(path).await;

        let _ = ctx.register_table("solar", Arc::new(t));
        // -19.6875,
        // 26.4312280645064

        let sql = r#"SELECT raquet_pixel(band_1,cast(32 as bigint),cast(17 as bigint)) from solar where block<>0 ;"#;
        println!("{:?}", sql);

        let df = ctx.sql(sql).await.unwrap();
        println!("{:?}", df.count().await);
    }
}
