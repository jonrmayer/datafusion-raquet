use std::sync::{Arc, OnceLock};

use arrow_array::builder::Int32Builder;
use arrow_array::cast::AsArray;
use arrow_array::types::{Float64Type, Int64Type};
use arrow_array::{ArrayRef, StructArray};
use arrow_schema::{DataType, Field, FieldRef, Fields};

use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
     Volatility,
};

use itertools::multizip;

use crate::error::{RaquetDataFusionError, RaquetDataFusionResult};

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
    fn data_type(&self) -> DataType {
        let values_fields = vec![
            Field::new("pixel_x", DataType::Int32, false),
            Field::new("pixel_y", DataType::Int32, false),
        ];
        DataType::Struct(values_fields.into())
    }
    fn to_field<N: Into<String>>(&self, name: N, nullable: bool) -> Field {
        Field::new(name, self.data_type(), nullable)
    }
}

impl Default for QuadBinToPixelXY {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for QuadBinToPixelXY {
    fn name(&self) -> &str {
        "quadbin_pixel_xy"
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

fn build_cell_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<StructArray> {
    let lon = arrays[0].as_primitive::<Float64Type>();
    let lat = arrays[1].as_primitive::<Float64Type>();
    let resolution = arrays[2].as_primitive::<Int64Type>();
    let tile_size = arrays[3].as_primitive::<Int64Type>();

    let mut pixel_x_builder = Int32Builder::new();
    let mut pixel_y_builder = Int32Builder::new();

    for (lon, lat, resolution, tile_size) in multizip((lon, lat, resolution, tile_size)) {
        let pc = lonlat_to_pixel(
            lon.unwrap(),
            lat.unwrap(),
            resolution.unwrap() as i8,
            tile_size.unwrap() as i16,
        );
        pixel_x_builder.append_value(pc.pixel_x as i32);
        pixel_y_builder.append_value(pc.pixel_y as i32);
    }

    let values_fields = vec![
        Field::new("pixel_x", DataType::Int32, false),
        Field::new("pixel_y", DataType::Int32, false),
    ];

    let fields = Fields::from(values_fields);

    let arrays: Vec<ArrayRef> = vec![
        Arc::new(pixel_x_builder.finish()),
        Arc::new(pixel_y_builder.finish()),
    ];

    let nulls = None;
    let arr = StructArray::new(fields, arrays, nulls);
    Ok(arr)
}

#[cfg(test)]
mod tests {
    use datafusion::prelude::SessionContext;

    use super::*;

    #[tokio::test]
    async fn test_quadbin_pixel_xy() {
        let ctx = SessionContext::new();
        ctx.register_udf(QuadBinToPixelXY::default().into());
        let sql = r#"SELECT quadbin_pixel_xy(0.0, 0.0, 4, 256) pixel;"#;
        println!("{:?}", sql);

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }
}
