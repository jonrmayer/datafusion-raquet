use std::sync::{Arc, OnceLock};

use crate::error::RaquetDataFusionResult;

use arrow_array::builder::Float64Builder;
use arrow_array::cast::AsArray;
use arrow_array::cast::as_string_array;
use arrow_array::types::Int64Type;
use arrow_array::{ArrayRef, BinaryArray, Float64Array};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

use itertools::multizip;

use rastertile_schema::Metadata as RMetadata;

use quadbin_schema::Metadata as QMetadata;

use quadbin_rs::lonlat_to_pixel;

use quadbin_geo_rs::wkt_to_lonlat;

use rastertile_rs::Operations;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct RaquetValue {
    signature: Signature,
}

impl RaquetValue {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(
                vec![DataType::Int64, DataType::Binary, DataType::Utf8],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for RaquetValue {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for RaquetValue {
    fn name(&self) -> &str {
        "raquet_value"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        Err(DataFusionError::Internal("return_type".to_string()))
    }

    fn return_field_from_args(&self, _args: ReturnFieldArgs) -> Result<FieldRef> {
        Ok(Arc::new(Field::new("", DataType::Float64, false)))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let quadbin_metadata = QMetadata::try_from(args.arg_fields[0].as_ref()).unwrap_or_default();
        let raster_metadata = RMetadata::try_from(args.arg_fields[1].as_ref()).unwrap_or_default();
        let arrays = ColumnarValue::values_to_arrays(&args.args)?;
        let cell_arr = build_cell_array(arrays, quadbin_metadata, raster_metadata)?;
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
// fn geocells(metadata: QMetadata, wkt: &String) -> RaquetDataFusionResult<Expr> {
//     let resolution = metadata.max_zoom().clone();
//     let geolist = GeoCells::new(wkt.clone(), resolution as i8)
//         .intersecting_cells()?
//         .iter()
//         .map(|x| lit(*x))
//         .collect();
//     let expr = col("block").in_list(geolist, false);

//     Ok(expr)
// }
fn build_cell_array(
    arrays: Vec<ArrayRef>,
    qmetadata: QMetadata,
    rmetadata: RMetadata,
) -> RaquetDataFusionResult<Float64Array> {
    let cell_array = arrays[0].as_primitive::<Int64Type>();
    let binary_array = arrays[1]
        .as_any()
        .downcast_ref::<BinaryArray>()
        .expect("cast failed");

    let wkt_array = as_string_array(&arrays[2]);

    let mut out_builder = Float64Builder::new();
    let ops: Operations = Operations::new(rmetadata.inner());

    for (cell, binary, wkt) in multizip((cell_array, binary_array, wkt_array)) {
        let lonlat = wkt_to_lonlat(wkt.unwrap().to_string());
       
        let pixel = lonlat_to_pixel(
            lonlat.0,
            lonlat.1,
            qmetadata.max_zoom().clone() as i8,
            rmetadata.tile_size() as i16,
        );
       
        let value = ops.getpixel(binary, pixel.pixel_x as u64, pixel.pixel_y as u64)?;
        out_builder.append_value(value);
    }

    let point_arr = out_builder.finish();

    Ok(point_arr)
}
#[cfg(test)]
mod tests {

    use super::*;
    use crate::RaquetTable;
    use crate::udf::quadbin::QuadBinToPixelXY;
    use crate::views::ReadRaquetAt;
    use datafusion::prelude::{SessionConfig, SessionContext};

    #[tokio::test]
    async fn test_raquet_pixel() {
        let path =
            "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet"
                .to_string();

        let ctx =
            SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

        ctx.register_udf(RaquetValue::default().into());
        ctx.register_udf(QuadBinToPixelXY::default().into());
        ctx.register_udtf("read_raquet_at", Arc::new(ReadRaquetAt {}));
        let t = RaquetTable::from_path(path).await;

        let _ = ctx.register_table("solar", Arc::new(t));
        // -19.6875,
        // 26.4312280645064

        let sql = r#"select raquet_value(cast(block as bigint),band_1,'POINT(-3.7038 40.4168)') p from solar where block = 5229757908543078399"#;
        println!("{:?}", sql);

        let df = ctx.sql(sql).await.unwrap();
         df.clone().show().await.unwrap();
        println!("{:?}", df.count().await);
    }
}
