use std::any::Any;
use std::sync::{Arc, OnceLock};

use arrow_array::builder::{Int32Builder, ListBuilder, StringBuilder, UInt8Builder, UInt64Builder};
use arrow_array::cast::as_string_array;

use arrow_array::{ArrayRef, Int64Array, ListArray, StructArray, UInt64Array};
use arrow_schema::{DataType, Field, FieldRef, Fields};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

use crate::error::RaquetDataFusionResult;

use crate::{raquet_band_metadata, raquet_format_from_str, raquet_quadbin_metadata};

use quadbin_geo_rs::GeoCells;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct QuadbinMetadata {
    signature: Signature,
}

impl QuadbinMetadata {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(vec![DataType::Utf8], Volatility::Immutable),
        }
    }

    // pub tile_size: usize,
    // pub binary_type: BinaryType,
    // pub data_type: RasterDataType,
    // pub no_data: String,
    // pub compression: CompressionFormat,
    // pub bands: Option<Vec<String>>,
    fn data_type(&self) -> DataType {
        let values_fields = vec![
            Field::new("min_zoom", DataType::Int32, false),
            Field::new("max_zoom", DataType::Int32, false),
        ];
        DataType::Struct(values_fields.into())
    }
    fn to_field<N: Into<String>>(&self, name: N, nullable: bool) -> Field {
        Field::new(name, self.data_type(), nullable)
    }
}

impl Default for QuadbinMetadata {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for QuadbinMetadata {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn name(&self) -> &str {
        "quadbin_metadata"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        Err(DataFusionError::Internal("return_type".to_string()))
    }

    fn return_field_from_args(&self, args: ReturnFieldArgs) -> Result<FieldRef> {
        Ok(Arc::new(self.to_field("", false)))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let arrays = ColumnarValue::values_to_arrays(&args.args)?;
        let cell_arr = build_cell_array(arrays)?;
        let array_ref: ArrayRef = Arc::new(cell_arr);
        Ok(ColumnarValue::Array(array_ref))
    }

    fn documentation(&self) -> Option<&Documentation> {
        Some(DOCUMENTATION.get_or_init(|| {
            Documentation::builder(
                DOC_SECTION_OTHER,
                "Return a metdata struct for a band",
                "quadbin_metadata(metadata)",
            )
            .with_argument("metadata", "metadata value")
            .build()
        }))
    }
}

fn build_cell_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<StructArray> {
    let metadata_array = as_string_array(&arrays[0]);

    let mut min_zoom_builder = Int32Builder::new();
    let mut max_zoom_builder = Int32Builder::new();

    for metadata in metadata_array.iter() {
        let rqm = raquet_quadbin_metadata(metadata.unwrap());

        min_zoom_builder.append_value(rqm.min_zoom);
        max_zoom_builder.append_value(rqm.max_zoom);
    }

    let values_fields = vec![
        Field::new("min_zoom", DataType::Int32, false),
        Field::new("max_zoom", DataType::Int32, false),
    ];

    let fields = Fields::from(values_fields);

    let arrays: Vec<ArrayRef> = vec![
        Arc::new(min_zoom_builder.finish()),
        Arc::new(max_zoom_builder.finish()),
    ];
    let nulls = None;
    let arr = StructArray::new(fields, arrays, nulls);

    Ok(arr)

    // for (metadata, wkt) in metadata_array.iter().zip(wkt_array.iter()) {
    //     let (min, max) = raquet_quadbin_minmax(metadata.unwrap());
    //     let geocells = GeoCells::new(wkt.unwrap().to_string(), max as i8).intersecting_cells()?;
    //     let bounding = UInt64Array::from(geocells);

    //     builder.append_value(&bounding);
    // }

    // let point_arr = builder.finish();

    // Ok(point_arr)
}

// #[cfg(test)]
// mod tests {

//     use crate::RaquetTable;
//     use crate::register;
//     use crate::views::ReadRaquetMetadata;
//     use datafusion::prelude::{SessionConfig, SessionContext};

//     use super::*;
//     pub async fn setup_local() -> SessionContext {
//         let path =
//         "file:///home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet"
//             .to_string();

//         let mut ctx =
//             SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

//         // register(&mut ctx);
//         ctx.register_udf(QuadbinMetadata::default().into());

//         let t = RaquetTable::from_path(path).await;

//         let _ = ctx.register_table("solar", Arc::new(t));
//         ctx
//     }

//     #[tokio::test]
//     async fn test_intersects() {
//         let ctx = setup_local().await;

//         let sql = r###"
//         with m as (
//             select metadata from solar where block=0

//         )

//         select quadbin_metadata(m.metadata) as bmeta from m
       

   
//     "###;

//         let df = ctx.sql(sql).await.unwrap();
//         df.show().await.unwrap();
//     }
// }
