use std::sync::Arc;
use std::collections::HashMap;
use arrow::array::{Array, ArrayRef, BinaryArray, AsArray};
use arrow::datatypes::{DataType as ArrowDataType, Field};
use datafusion_common::{exec_err, Result, ScalarValue};
use datafusion_expr::{ColumnarValue, ScalarUDFImpl, Signature, Volatility, UdfArgs, Expr};
use datafusion_expr::return_field::ReturnField;

use rastertile_schema::metadata::BandMetadata;
use rastertile::compression::{CompressConfiguration, GzipConfig, JpegConfig, WebpConfig};

#[derive(Debug)]
pub struct CompressUdf {
    signature: Signature,
}

impl CompressUdf {
    pub fn new() -> Self {
        Self {
            signature: Signature::one_of(
                vec![
                    vec![ArrowDataType::Binary, ArrowDataType::Utf8],
                    vec![ArrowDataType::Binary, ArrowDataType::Utf8, ArrowDataType::Int32],
                    vec![ArrowDataType::Binary, ArrowDataType::Utf8, ArrowDataType::Int32, ArrowDataType::Int32, ArrowDataType::Int32],
                    vec![ArrowDataType::Binary, ArrowDataType::Utf8, ArrowDataType::Int32, ArrowDataType::Int32, ArrowDataType::Boolean, ArrowDataType::Int32],
                ],
                Volatility::Immutable,
            ),
        }
    }
}

impl ScalarUDFImpl for CompressUdf {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn name(&self) -> &str { "raquet_compress" }
    fn signature(&self) -> &Signature { &self.signature }
    fn return_type(&self, _arg_types: &[ArrowDataType]) -> Result<ArrowDataType> {
        Ok(ArrowDataType::Binary)
    }

    /// Modifies and attaches updated compression layout rules inside your schema metadata
    fn return_field_impl(&self, args: UdfArgs) -> Result<ReturnField> {
        let mut metadata_map = HashMap::new();

        let mut format_str = "none".to_string();
        if args.args.len() >= 2 {
            if let Expr::Literal(ScalarValue::Utf8(Some(f))) = &args.args[1] {
                format_str = f.to_lowercase();
            }
        }

        // Parse and check if an existing upstream schema field was supplied
        if let Some(input_field) = args.input_types.first() {
            if let Some(upstream_json) = input_field.metadata().get("rastertile.band_config") {
                if let Ok(mut band_metadata) = serde_json::from_str::<BandMetadata>(upstream_json) {
                    // Update the metadata layer configuration to reflect the current codec state
                    band_metadata.compression = format_str.clone();
                    band_metadata.binary_type = if format_str == "none" { "uncompressed".to_string() } else { "compressed".to_string() };
                    
                    metadata_map.insert("rastertile.band_config".to_string(), serde_json::to_string(&band_metadata).unwrap_or_default());
                }
            }
        }

        let field = Field::new(self.name(), ArrowDataType::Binary, true)
            .with_metadata(metadata_map);

        Ok(ReturnField::new(field))
    }

    fn invoke(&self, args: &[ColumnarValue]) -> Result<ColumnarValue> {
        if args.len() < 2 {
            return exec_err!("raquet_compress requires at least 2 arguments (data, format)");
        }

        let arrays = ColumnarValue::values_to_arrays(args)?;
        let binary_array = arrays.as_binary::<i32>();
        let format_array = arrays.as_string::<i32>();

        let mut builder = arrow::array::BinaryBuilder::with_capacity(binary_array.len(), binary_array.len() * 256);

        for i in 0..binary_array.len() {
            if binary_array.is_null(i) {
                builder.append_null();
                continue;
            }

            let raw_bytes = binary_array.value(i);
            let format_str = format_array.value(i).to_lowercase();

            let config = match format_str.as_str() {
                "none" => CompressConfiguration::None,
                "gzip" => {
                    let level = if arrays.len() >= 3 {
                        arrays.as_primitive::<arrow::datatypes::Int32Type>().value(i) as u32
                    } else { 6 };
                    CompressConfiguration::Gzip(GzipConfig { level })
                }
                "jpeg" => {
                    if arrays.len() < 5 { return exec_err!("JPEG configurations require width, height, and quality parameters"); }
                    let width = arrays.as_primitive::<arrow::datatypes::Int32Type>().value(i) as u32;
                    let height = arrays.as_primitive::<arrow::datatypes::Int32Type>().value(i) as u32;
                    let quality = arrays.as_primitive::<arrow::datatypes::Int32Type>().value(i) as u8;
                    CompressConfiguration::Jpeg(JpegConfig { width, height, quality })
                }
                "webp" => {
                    if arrays.len() < 6 { return exec_err!("WebP configurations require width, height, lossless, and quality parameters"); }
                    let width = arrays.as_primitive::<arrow::datatypes::Int32Type>().value(i) as u32;
                    let height = arrays.as_primitive::<arrow::datatypes::Int32Type>().value(i) as u32;
                    let lossless = arrays.as_boolean().value(i);
                    let quality = arrays.as_primitive::<arrow::datatypes::Int32Type>().value(i) as u8;
                    CompressConfiguration::Webp(WebpConfig { width, height, lossless, quality })
                }
                _ => return exec_err!("Unknown compression format configuration requested: {}", format_str),
            };

            let compressed_blob = rastertile::compression::compress(raw_bytes, &config)
                .map_err(|e| datafusion_common::DataFusionError::Execution(format!("Codec crash: {:?}", e)))?;

            builder.append_value(&compressed_blob);
        }

        Ok(ColumnarValue::Array(Arc::new(builder.finish())))
    }
}
