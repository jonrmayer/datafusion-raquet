use std::any::Any;
use std::sync::{Arc, OnceLock};

use arrow_array::builder::{
    ArrayBuilder, Float64Builder, ListBuilder, StructBuilder, UInt64Builder,
};
use arrow_array::cast::AsArray;
use arrow_array::types::{Float64Type, Int64Type};
use arrow_array::{Array, ArrayRef, GenericListArray, ListArray, StructArray, UInt64Array};
use arrow_schema::{DataType, Field, FieldRef, Fields};

use arrow_convert::{
    ArrowDeserialize, ArrowField, ArrowSerialize, deserialize::TryIntoCollection,
    serialize::TryIntoArrow,
};

use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    TypeSignature, Volatility,
};

use crate::error::{RaquetDataFusionError, RaquetDataFusionResult};

use crate::udf::quadbin::converter::{Abbox, LonLat, Pixel};
use quadbin_rs::lonlat_to_pixel;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct QuadBinToPixelXY {
    signature: Signature,
}

impl QuadBinToPixelXY {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(
                vec![
                    DataType::Float64,
                    DataType::Float64,
                    DataType::Int64,
                    DataType::Int64,
                ],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for QuadBinToPixelXY {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for QuadBinToPixelXY {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> &str {
        "quadbin_pixel_xy"
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
                "Return a Pixel Struct from lon, lat, res, tile_size ",
                "quadbin_pixel_xy(lon, lat, res, tile_size) ",
            )
            .with_argument("lon", "lon value")
            .with_argument("lat", "lat value")
            .with_argument("res", "res value")
            .with_argument("tile_size", "tile_size value")
            .build()
        }))
    }
}

fn return_field_impl(_args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    let pixel_x = Field::new("pixel_x", DataType::Int32, false);
    let pixel_y = Field::new("pixel_y", DataType::Int32, false);

    let fields = Fields::from(vec![pixel_x, pixel_y]);
    let pixel = Field::new_struct("", fields, false);
    // let item_field = Arc::new(bbox.clone());
    Ok(Arc::new(pixel))
}

use itertools::multizip;

fn build_cell_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<StructArray> {
    let lon = arrays[0].as_primitive::<Float64Type>();
    let lat = arrays[1].as_primitive::<Float64Type>();
    let resolution = arrays[2].as_primitive::<Int64Type>();
    let tile_size = arrays[3].as_primitive::<Int64Type>();
    let mut vcells: Vec<Pixel> = vec![];
    for (lon, lat, resolution, tile_size) in multizip((lon, lat, resolution, tile_size)) {
        let pc = lonlat_to_pixel(lon.unwrap(), lat.unwrap(), resolution.unwrap() as i8, tile_size.unwrap() as i16);
        let pixel:Pixel = Pixel::new(pc.pixel_x as i32, pc.pixel_y as i32);
        vcells.push(pixel);
    }

    let box_array: ArrayRef = vcells.try_into_arrow().unwrap();
    let struct_array = box_array
        .as_any()
        .downcast_ref::<arrow::array::StructArray>()
        .unwrap();
    Ok(struct_array.clone())
}

#[cfg(test)]
mod tests {
    use datafusion::prelude::SessionContext;

    use super::*;

    #[tokio::test]
    async fn test_quadbin_to_children() {
        let ctx = SessionContext::new();
        ctx.register_udf(QuadBinToPixelXY::default().into());
        let sql = r#"SELECT quadbin_to_bbox_mercator(5256690695657226239) ;"#;
        println!("{:?}", sql);

        let df = ctx.sql(sql).await.unwrap();
        // df.show();
        let batches = df.collect().await.unwrap();
        // let column = batches[0].column(0);
        // // let string_arr = column.as_string_view();

        // let val = column.as_list(0).value(0);
        // println!("{:?}", val);
    }

    #[tokio::test]
    async fn test_quadbin_to_parent_resolution() {
        let ctx = SessionContext::new();
        ctx.register_udf(QuadBinToPixelXY::default().into());
        let sql = r#"SELECT quadbin_to_children(5256690695657226239,13) cell;"#;
        println!("{:?}", sql);

        let df = ctx.sql(sql).await.unwrap();
        let batches = df.collect().await.unwrap();
        let column = batches[0].column(0);
        // let string_arr = column.as_string_view();

        // let val = column.as_primitive::<UInt64Type>().value(0);
        // println!("{:?}", val);
    }
}
