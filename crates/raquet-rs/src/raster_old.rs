use arrow::array::{Array, BinaryArray, Float64Array, BinaryBuilder};
use arrow::error::ArrowError;
// use rastertile_rs::{array::NativeType, operations::Operations, BandMetadata, DataType};
use rastertile_rs::{Operations,Metadata};

/// `RasterOrchestrator` maintains an active tile operation configuration state,
/// processing decompression and decoding pipelines directly across native Apache Arrow arrays.
pub struct RasterOrchestrator {
    ops: Operations,
   
}

impl RasterOrchestrator {
    /// Instantiates a new `RasterOrchestrator` state pipeline bound to explicit band metadata traits.
    pub fn new(metadata: Metadata) -> Self {
        Self {
            ops: Operations::new(metadata),
            
        }
    }

    /// Vectorized decompression pipeline transforming a `BinaryArray` of compressed blobs 
    /// straight into an uncompressed `BinaryArray`.
    pub fn decompress_batch(&self, band_array: &BinaryArray) -> Result<BinaryArray, ArrowError> {
        let num_rows = band_array.len();
        // Pre-size your builder with structural capacity parameters upfront to prevent reallocation loops
        let mut builder = BinaryBuilder::with_capacity(num_rows, band_array.value_data().len() * 4);

        for i in 0..num_rows {
            if band_array.is_null(i) {
                builder.append_null();
                continue;
            }

            let compressed_bytes = band_array.value(i);
            
            // Invoke the instance method on the pre-initialized Operations context
            match self.ops.decompress_bytes(compressed_bytes, &self.metadata.compression) {
                Ok(decompressed_bytes) => builder.append_value(&decompressed_bytes),
                Err(err) => {
                    return Err(ArrowError::ComputeError(format!(
                        "Batch decompression failure at row {i}: {err}"
                    )));
                }
            }
        }

        Ok(builder.finish())
    }

    /// Fully unrolls a compressed `BinaryArray` batch and decodes raw bytes into flat arrays,
    /// casting elements to a standard Arrow `Float64Array` depending on the metadata type traits.
    pub fn decode_batch(&self, band_array: &BinaryArray) -> Result<Float64Array, ArrowError> {
        let num_rows = band_array.len();
        
        // 1. Inflate all columns in the batch through your uncompressed byte buffer channel
        let uncompressed_batch = self.decompress_batch(band_array)?;
        
        // 2. Pre-size the destination Float64 builder window context
        let mut builder = Float64Array::builder(num_rows);

        for i in 0..num_rows {
            if uncompressed_batch.is_null(i) {
                builder.append_null();
                continue;
            }

            let decompressed_bytes = uncompressed_batch.value(i);

            // 3. Match against your explicit structural primitive types
            // For each variation, unbox via zero-copy buffer casting and collect into the array builder
            match self.metadata.data_type {
                DataType::Float64 => {
                    match self.ops.decode_buffer::<f64>(&decompressed_bytes) {
                        Ok(values) => {
                            // Extract first pixel or process the array context depending on layout scope
                            if let Some(&first_val) = values.first() {
                                builder.append_value(first_val);
                            } else {
                                builder.append_null();
                            }
                        }
                        Err(err) => return Err(ArrowError::ComputeError(format!("Decoding error: {err}"))),
                    }
                }
                DataType::Int16 => {
                    match self.ops.decode_buffer::<i16>(&decompressed_bytes) {
                        Ok(values) => {
                            if let Some(&first_val) = values.first() {
                                builder.append_value(first_val as f64); // Cast seamlessly to Float64 output bounds
                            } else {
                                builder.append_null();
                            }
                        }
                        Err(err) => return Err(ArrowError::ComputeError(format!("Decoding error: {err}"))),
                    }
                }
                // Extend with any remaining data types supported by your rastertile-rs primitive mappings
                _ => {
                    return Err(ArrowError::ComputeError(format!(
                        "Unsupported primitive data layout variant: {:?}", 
                        self.metadata.data_type
                    )));
                }
            }
        }

        Ok(builder.finish())
    }
}