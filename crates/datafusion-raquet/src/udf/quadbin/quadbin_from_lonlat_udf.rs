use std::any::Any;
use std::sync::{Arc, OnceLock};



use arrow::datatypes::{Float64Type,  Int8Type};

use arrow::array::{ArrayRef,  UInt64Array,  AsArray};
use arrow_schema::{DataType, Field, FieldRef};
use datafusion::error::{DataFusionError, Result};
use datafusion_common::internal_err;
use datafusion::logical_expr::scalar_doc_sections::DOC_SECTION_OTHER;
use datafusion::logical_expr::{
    ColumnarValue, Documentation, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDFImpl, Signature,
    Volatility,
};

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
        let res_slice = res_array.as_primitive::<Int8Type>().values();

        // 1. Validate that the resolution scalar is uniform across the current batch chunk
        if res_slice.is_empty() {
            return Ok(ColumnarValue::Array(
                Arc::new(UInt64Array::from(vec![0u64; 0])) as ArrayRef,
            ));
        }
        let target_resolution = res_slice[0];
        if !res_slice.iter().all(|&r| r == target_resolution) {
            return internal_err!(
                "Raquet Error: Vectorized batch resolution settings must be uniform across array blocks"
            );
        }

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
