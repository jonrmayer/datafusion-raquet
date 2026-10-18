use std::any::Any;
use std::sync::{Arc, OnceLock};

use arrow::datatypes::{Float64Type, Int8Type};

use arrow::array::{ArrayRef, AsArray, UInt64Array};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};
use datafusion_common::internal_err;

use crate::error::RaquetDataFusionResult;

use raquet_rs::RaquetOrchestrator;

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct QuadbinFromLonLatUDF {
    signature: Signature,
}

impl QuadbinFromLonLatUDF {
    pub fn new() -> Self {
        Self {
            signature: Signature::exact(
                vec![DataType::Float64, DataType::Float64, DataType::Int64],
                Volatility::Immutable,
            ),
        }
    }
}

impl Default for QuadbinFromLonLatUDF {
    fn default() -> Self {
        Self::new()
    }
}

static DOCUMENTATION: OnceLock<Documentation> = OnceLock::new();

impl ScalarUDFImpl for QuadbinFromLonLatUDF {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn name(&self) -> &str {
        "quadbin_from_lonlat"
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    fn return_type(&self, _arg_types: &[DataType]) -> Result<DataType> {
        Err(DataFusionError::Internal("return_type".to_string()))
    }

    fn return_field_from_args(&self, _args: ReturnFieldArgs) -> Result<FieldRef> {
        Ok(Arc::new(Field::new("", DataType::UInt64, false)))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        // Extract total evaluated batch execution scale length
        let len = args.number_rows;

        if args.args.len() != 3 {
            return internal_err!(
                "Raquet Error: quadbin_from_lonlat expects exactly 3 arguments (lon, lat, resolution)"
            );
        }

        // Coerce the input value arguments safely into flat, zero-copy array views
        let lons_array = args.args[0].clone().into_array(len)?;
        let lats_array = args.args[1].clone().into_array(len)?;
        let res_array = args.args[2].clone().into_array(len)?;

        let lons = lons_array.as_primitive::<Float64Type>().values();
        let lats = lats_array.as_primitive::<Float64Type>().values();
        let res_int64_slice = res_array
            .as_primitive::<arrow::datatypes::Int64Type>()
            .values();

        //  Short-circuit cleanly if DataFusion forwards an empty slice block
        if res_int64_slice.is_empty() {
            return Ok(ColumnarValue::Array(
                Arc::new(UInt64Array::from(vec![0u64; 0])) as ArrayRef,
            ));
        }

        // 2. Extract the first row element as our primary target value candidate
        let raw_resolution = res_int64_slice[0];

        // 3. Structural Validation: Ensure all elements in this batch block are uniform
        if !res_int64_slice.iter().all(|&r| r == raw_resolution) {
            return internal_err!(
                "Raquet Error: Vectorized batch resolution settings must be uniform across array blocks"
            );
        }

        // 4. Safe Boundary Guard: Enforce strict Quadbin engineering limits before casting
        if !(0..=26).contains(&raw_resolution) {
            return internal_err!(
                "Raquet Error: Quadbin indexing resolution {} is out of valid bounds (0 to 26)",
                raw_resolution
            );
        }

        // 5. Explicit Safe Coercion: Safely cast the validated i64 scalar to a native i8
        let target_resolution = raw_resolution as i8;

        

        // 2. Pre-allocate flat memory array buffer space upfront for the result elements
        let mut output_cells = vec![0u64; len];

        // 3. Delegate execution directly down to your un-gated raquet-rs orchestrator vector loop
        RaquetOrchestrator::batch_lonlat_to_cells(lons, lats, target_resolution, &mut output_cells)
            .map_err(|e| datafusion::common::DataFusionError::Execution(e.to_string()))?;

        // 4. Return the flat populated primitive Arrow array
        let result_array = UInt64Array::from(output_cells);
        Ok(ColumnarValue::Array(Arc::new(result_array) as ArrayRef))
        // }
    }

    fn documentation(&self) -> Option<&Documentation> {
        Some(DOCUMENTATION.get_or_init(|| {
            Documentation::builder(
                DOC_SECTION_OTHER,
                "Return a QUADBIN cell from a LonLat and Resolution(lon,lat,resolution).",
                "quadbin_from_latlon(lon,lat,resolution)",
            )
            .with_argument("lon", "lon value")
            .with_argument("lat", "lat value")
            .with_argument("resolution", "resolution value")
            .build()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Float64Array, Int64Array};

    #[test]
    fn test_quadbin_from_lonlat_udf_success() {
        let lons = vec![-0.1278, -74.0060];
        let lats = vec![51.5074, 40.7128];
        let resolutions = vec![4i64, 4i64]; // Matches DataType::Int64 signature definition
        let num_rows = lons.len();

        let lon_array = Arc::new(Float64Array::from(lons)) as ArrayRef;
        let lat_array = Arc::new(Float64Array::from(lats)) as ArrayRef;
        let res_array = Arc::new(Int64Array::from(resolutions)) as ArrayRef;

        let lon_field = Arc::new(Field::new("lon", DataType::Float64, false));
        let lat_field = Arc::new(Field::new("lat", DataType::Float64, false));
        let res_field = Arc::new(Field::new("resolution", DataType::Int64, false));
        let return_field = Arc::new(Field::new("", DataType::UInt64, false));

        // Construct standard DataFusion 53 execution arguments wrapper block
        let args = ScalarFunctionArgs {
            args: vec![
                ColumnarValue::Array(lon_array),
                ColumnarValue::Array(lat_array),
                ColumnarValue::Array(res_array),
            ],
            arg_fields: vec![lon_field, lat_field, res_field],
            number_rows: num_rows,
            return_field,
            config_options: Arc::new(datafusion::config::ConfigOptions::default()),
        };

        let udf = QuadbinFromLonLatUDF::new();
        let result = udf.invoke_with_args(args);

        assert!(result.is_ok());
        if let ColumnarValue::Array(array_ref) = result.unwrap() {
            let cells = array_ref.as_primitive::<arrow::datatypes::UInt64Type>();
            assert_eq!(cells.len(), num_rows);
            assert_ne!(cells.value(0), 0);
            assert_ne!(cells.value(1), 0);
        } else {
            panic!("Expected ColumnarValue::Array return payload variant");
        }
    }

    #[test]
    fn test_quadbin_from_lonlat_udf_mismatched_arguments() {
        let lons = vec![0.0];
        let lon_array = Arc::new(Float64Array::from(lons)) as ArrayRef;

        let args = ScalarFunctionArgs {
            args: vec![ColumnarValue::Array(lon_array)], // Missing lat and resolution
            arg_fields: vec![Arc::new(Field::new("lon", DataType::Float64, false))],
            number_rows: 1,
            return_field: Arc::new(Field::new("", DataType::UInt64, false)),
            config_options: Arc::new(datafusion::config::ConfigOptions::default()),
        };

        let udf = QuadbinFromLonLatUDF::new();
        let result = udf.invoke_with_args(args);
        assert!(result.is_err());
    }
}
