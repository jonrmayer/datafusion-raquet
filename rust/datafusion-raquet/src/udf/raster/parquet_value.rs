use std::sync::{Arc, OnceLock};
use std::any::Any;
use crate::error::RaquetDataFusionResult;
use arrow_array::builder::{Float64Builder, ListBuilder};
use arrow_array::{
    ArrayRef, BinaryArray, BinaryViewArray, Float64Array, ListArray, StringViewArray,
};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

use rastertile_schema::Metadata;

use itertools::multizip;

use crate::{raquet_band_metadata, raquet_format_from_str, raquet_quadbin_metadata};
use quadbin_rs::lonlat_to_pixel;
use rastertile_rs::Operations;

use quadbin_geo_rs::wkt_to_lonlat;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct ParquetValue {
    signature: Signature,
}

impl ParquetValue {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(
                vec![DataType::BinaryView, DataType::Utf8View, DataType::Utf8View],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for ParquetValue {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for ParquetValue {
         fn as_any(&self) -> &dyn Any {
        self
    }
    fn name(&self) -> &str {
        "parquet_value"
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

        let binary_name = binary_field.name();
        let arrays = ColumnarValue::values_to_arrays(&args.args)?;
        let cell_arr = build_cell_array(binary_name, arrays)?;
        let array_ref: ArrayRef = Arc::new(cell_arr);
        Ok(ColumnarValue::Array(array_ref))
    }

    fn documentation(&self) -> Option<&Documentation> {
        Some(DOCUMENTATION.get_or_init(|| {
            Documentation::builder(
                DOC_SECTION_OTHER,
                "Return a decoded binary from an encoded binary.",
                "parquet_value(band,wkt,metadata)",
            )
            .with_argument("band", "band value")
            .with_argument("wkt", "wkt value")
            .with_argument("metadata", "metadata value")
            .build()
        }))
    }
}

fn return_field_impl(_args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    Ok(Arc::new(Field::new("", DataType::Float64, false)))
}

fn build_cell_array(
    binary_name: &String,
    arrays: Vec<ArrayRef>,
) -> RaquetDataFusionResult<Float64Array> {
    let in_binary = arrays[0]
        .as_any()
        .downcast_ref::<BinaryViewArray>()
        .expect("cast failed");
    let wkt_array = arrays[1]
        .as_any()
        .downcast_ref::<StringViewArray>()
        .expect("cast failed");
    let metadata_array = arrays[2]
        .as_any()
        .downcast_ref::<StringViewArray>()
        .expect("cast failed");
    let mut out_builder = Float64Builder::new();
    for (binary, wkt, metadata) in multizip((in_binary, wkt_array, metadata_array)) {
        let rcm = raquet_band_metadata(binary_name, metadata.unwrap());
        let ops: Operations = Operations::new(rcm.clone());
        let qcm = raquet_quadbin_metadata(metadata.unwrap());
        let lonlat = wkt_to_lonlat(wkt.unwrap().to_string());

        let pixel = lonlat_to_pixel(
            lonlat.0,
            lonlat.1,
            qcm.max_zoom as i8,
            rcm.tile_size() as i16,
        );
        let value = ops.getpixel(binary, pixel.pixel_x as u64, pixel.pixel_y as u64)?;
        out_builder.append_value(value);
    }

    let point_arr = out_builder.finish();

    Ok(point_arr)
}

// #[cfg(test)]
// mod tests {

//     use super::*;
//     use crate::RaquetTable;
//     use crate::udf::general::intersects::Intersects;
//     use datafusion::prelude::*;
//     use datafusion::prelude::{SessionConfig, SessionContext};

//     pub async fn setup_local_parquet() -> SessionContext {
//         let path =
//             "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet";

//         let ctx = SessionContext::new();
//         // SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

//         // register(&mut ctx);
//         ctx.register_udf(ParquetValue::default().into());
//         ctx.register_udf(Intersects::default().into());
//         let _ = ctx
//             .register_parquet("solar", path, ParquetReadOptions::default())
//             .await
//             .map_err(|e| {
//                 DataFusionError::Context(format!("Registering 'hits_raw' as {path}"), Box::new(e))
//             });

//         ctx
//     }

//     #[tokio::test]
//     async fn test_decode_tile_parquet() {
//         let ctx = setup_local_parquet().await;
//         let sql = r###"
//         with m as (
//             select metadata from solar where block=0

//         ),
//         data as (
//             select block,band_1  from solar
//             where block<>0 
//         ),
//         idata as (
//             select unnest(intersects('POINT(-3.7038 40.4168)',m.metadata)) indata from m
//         )

//         select parquet_value(data.band_1,'POINT(-3.7038 40.4168)',m.metadata) value from data,m,idata
        
//         where idata.indata=data.block
       
       

   
//     "###;

//         let df = ctx.sql(sql).await.unwrap();
//         df.show().await.unwrap();
//     }

//     #[tokio::test]
//     async fn test_native_interleaved_tile() {
//         let path =
//             "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet"
//                 .to_string();

//         let ctx =
//             SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

//         ctx.register_udf(ParquetValue::default().into());

//         let t = RaquetTable::from_path(path).await;

//         let _ = ctx.register_table("solar", Arc::new(t));

//         let sql = "select decode_tile(band_1) from solar where block<>0   ;";
//         let df = ctx.sql(sql).await.unwrap();
//         println!("{:?}", df.count().await);
//     }
// }
