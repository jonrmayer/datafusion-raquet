use std::sync::{Arc, OnceLock};

use crate::error::RaquetDataFusionResult;
use arrow_array::builder::GenericBinaryBuilder;
use arrow_array::cast::as_string_array;
use arrow_array::{ArrayRef, BinaryArray, BinaryViewArray, StringViewArray};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    TypeSignature, Volatility,
};

use datafusion::common::ScalarValue;

use rastertile_schema::{Metadata, RasterArrowType, RasterType};

use rastertile_rs::{CompressionFormat, Metadata as RasterMetadata, Operations};

use crate::udf::raster::utils::has_extension;
use crate::{raquet_band_metadata, raquet_format_from_str};

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct CastRaquet {
    signature: Signature,
}

impl CastRaquet {
    pub fn new() -> Self {
        Self {
            signature: Signature::one_of(
                vec![TypeSignature::Exact(vec![
                    DataType::BinaryView,
                    DataType::Utf8,
                    DataType::Utf8,
                    DataType::Utf8,
                    DataType::Utf8,
                    DataType::Utf8,
                ])],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for CastRaquet {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for CastRaquet {
    fn name(&self) -> &str {
        "cast_raquet"
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
                "Return a decompressed binary from an compressed binary.",
                "raquet_band_decompress(band_1,metadata)",
            )
            .with_argument("band", "band value")
            .with_argument("metadata", "optional metadata value")
            .build()
        }))
    }
}

fn return_field_impl(args: ReturnFieldArgs) -> RaquetDataFusionResult<FieldRef> {
    let binary_field = &args.arg_fields[0];

    let scalar_args = args.scalar_arguments;

    let tile_size_str = if let Some(ScalarValue::Utf8(Some(tile_size))) = scalar_args[1] {
        tile_size.clone()
    } else {
        "".to_string()
    };
    let binary_type_str = if let Some(ScalarValue::Utf8(Some(binary_type))) = scalar_args[2] {
        binary_type.clone()
    } else {
        "".to_string()
    };
    let data_type_str = if let Some(ScalarValue::Utf8(Some(data_type))) = scalar_args[3] {
        data_type.clone()
    } else {
        "".to_string()
    };
    let no_data_str = if let Some(ScalarValue::Utf8(Some(no_data))) = scalar_args[4] {
        no_data.clone()
    } else {
        "".to_string()
    };
    let compression_str = if let Some(ScalarValue::Utf8(Some(compression))) = scalar_args[5] {
        compression.clone()
    } else {
        "".to_string()
    };

    let rm = RasterMetadata::new_from_strings(
        tile_size_str,
        binary_type_str,
        data_type_str,
        no_data_str,
        compression_str,
        None,
    );
    let new_metadata = Metadata::new(
        rm.tile_size(),
        rm.binary_type(),
        rm.data_type(),
        rm.no_data(),
        rm.compression(),
        rm.bands(),
    );
    let metadata = Arc::new(new_metadata);
    // GeoArrowType:
    let raster_type = RasterType::new(metadata);
    let raster_arrow_type = match binary_field.data_type() {
        DataType::Binary => RasterArrowType::Raster(raster_type),
        DataType::LargeBinary => RasterArrowType::LargeRaster(raster_type),
        DataType::BinaryView => RasterArrowType::RasterView(raster_type),
        _ => unreachable!(),
    };

    Ok(Arc::new(
        raster_arrow_type.to_field(binary_field.name(), true),
    ))
}

fn build_cell_array(arrays: Vec<ArrayRef>) -> RaquetDataFusionResult<BinaryViewArray> {
    let in_binary = arrays[0]
        .as_any()
        .downcast_ref::<BinaryViewArray>()
        .expect("cast failed");

    Ok(in_binary.clone())
}

#[cfg(test)]
mod tests {
    use crate::RaquetTable;
    use crate::register;
    use crate::udf::raster::DecodeTile;
    use crate::views::ReadRaquetMetadata;
    use datafusion::prelude::*;
    use datafusion::prelude::{SessionConfig, SessionContext};

    use super::*;
    pub async fn setup_local() -> SessionContext {
        let path =
        "file:///home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet"
            .to_string();

        let mut ctx =
            SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

        // register(&mut ctx);
        ctx.register_udf(CastRaquet::default().into());

        let t = RaquetTable::from_path(path).await;

        let _ = ctx.register_table("solar", Arc::new(t));
        ctx
    }

    pub async fn setup_local_parquet() -> SessionContext {
        let path =
            "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet";

        let ctx = SessionContext::new();
        // SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

        // register(&mut ctx);
        ctx.register_udf(CastRaquet::default().into());
        ctx.register_udf(DecodeTile::default().into());
        let _ = ctx
            .register_parquet("solar", path, ParquetReadOptions::default())
            .await
            .map_err(|e| {
                DataFusionError::Context(format!("Registering 'hits_raw' as {path}"), Box::new(e))
            });

        ctx
    }

    #[tokio::test]
    async fn test_decompress_tile() {
        let path =
            "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet"
                .to_string();

        let ctx =
            SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

        ctx.register_udf(CastRaquet::default().into());
        let t = RaquetTable::from_path(path).await;

        let _ = ctx.register_table("solar", Arc::new(t));

        let sql = r#"SELECT raquet_band_decompress(band_1) from solar where block<>0  limit 1 ;"#;
        println!("{:?}", sql);

        let df = ctx.sql(sql).await.unwrap();
        println!("{:?}", df.count().await);
        // df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_decompress_tile2() {
        let ctx = setup_local().await;
        let sql = r###"


        select cast_raquet(band_1,'256', 'Separated', 'float32','NaN','gzip') from solar where block<>0 limit 1
       
       

   
    "###;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_decompress_tile_parquet() {
        let ctx = setup_local_parquet().await;
        let sql = r###"
        select decode_tile(cast_raquet(band_1,'256', 'Separated', 'float32','NaN','gzip')) band from solar where block<>0 limit 1
       
       

   
    "###;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }
}
