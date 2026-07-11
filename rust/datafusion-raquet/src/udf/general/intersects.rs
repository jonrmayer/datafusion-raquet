use std::sync::{Arc, OnceLock};
use std::any::Any;

use arrow_array::builder::{ListBuilder, UInt64Builder};
use arrow_array::cast::as_string_array;

use arrow_array::{ArrayRef, Int64Array, ListArray, UInt64Array};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

use crate::error::RaquetDataFusionResult;

use crate::{raquet_format_from_str, raquet_quadbin_metadata};

use quadbin_geo_rs::GeoCells;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Intersects {
    signature: Signature,
}

impl Intersects {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(
                vec![DataType::Utf8, DataType::Utf8],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for Intersects {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for Intersects {
      fn as_any(&self) -> &dyn Any {
        self
    }
    fn name(&self) -> &str {
        "intersects"
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
        let arrays = ColumnarValue::values_to_arrays(&args.args)?;
        let cell_arr = build_cell_array(arrays)?;
        let array_ref: ArrayRef = Arc::new(cell_arr);
        Ok(ColumnarValue::Array(array_ref))
    }

    fn documentation(&self) -> Option<&Documentation> {
        Some(DOCUMENTATION.get_or_init(|| {
            Documentation::builder(
                DOC_SECTION_OTHER,
                "Return a List[QUADBIN] of cell children from a wkt geometry with a resolution",
                "intersects(wkt,metadata)",
            )
            .with_argument("wkt", "wkt value")
            .with_argument("metadata", "metadata value")
            .build()
        }))
    }
}

fn return_field_impl(_args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    let list_field = Field::new_list_field(DataType::UInt64, true);
    let dt = DataType::List(Arc::new(list_field));
    let out_field: Field = Field::new("", dt, true);

    Ok(Arc::new(out_field))
}

fn build_cell_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<ListArray> {
    let wkt_array = as_string_array(&arrays[0]);
    let metadata_array = as_string_array(&arrays[1]);

    let values_builder = UInt64Builder::new();
    let mut builder = ListBuilder::new(values_builder);

    for (metadata, wkt) in metadata_array.iter().zip(wkt_array.iter()) {
        let qcm = raquet_quadbin_metadata(metadata.unwrap());
        let geocells = GeoCells::new(wkt.unwrap().to_string(), qcm.max_zoom as i8).intersecting_cells()?;
        let bounding = UInt64Array::from(geocells);

        builder.append_value(&bounding);
    }

    let point_arr = builder.finish();

    Ok(point_arr)
}

#[cfg(test)]
mod tests {

    use crate::RaquetTable;
    use crate::register;
    use crate::views::ReadRaquetMetadata;
    use datafusion::prelude::{SessionConfig, SessionContext};

    use super::*;
    pub async fn setup_local() -> SessionContext {
        let path =
        "file:///home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet"
            .to_string();

        let mut ctx =
            SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

        // register(&mut ctx);
        ctx.register_udf(Intersects::default().into());

        let t = RaquetTable::from_path(path).await;

        let _ = ctx.register_table("solar", Arc::new(t));
        ctx
    }

    #[tokio::test]
    async fn test_intersects() {
        let ctx = setup_local().await;

        let sql = r###"
        with m as (
            select metadata from solar where block=0

        ),
        data as (
            select unnest(intersects('POINT(-3.7038 40.4168)',m.metadata)) indata from m
        )

        select solar.block from data,solar
        where data.indata=solar.block
       

   
    "###;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }
}
