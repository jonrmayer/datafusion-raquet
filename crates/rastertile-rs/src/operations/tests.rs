#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::Metadata;
    use crate::DataType;
    // use crate::operations::encode_typed_array;
    use crate::operations::{
        Array, CompressConfiguration, Operations, OperationsResult, ToTypedArray, TypedArray,
    };
    // Minimal MockMetadata struct matching the required repository schema
    #[derive(Clone, Debug)]
    pub struct MockMetadata {
        pub samples: u32,
        pub tile_size: u32,
        pub data_type: crate::DataType,
    }

    impl MockMetadata {
        pub fn samples(&self) -> u32 {
            self.samples
        }
        pub fn tile_size(&self) -> u32 {
            self.tile_size
        }
        pub fn data_type(&self) -> crate::DataType {
            self.data_type
        }
    }

    // Minimal Operations implementation matching your API signatures
    pub struct TestOperations {
        metadata: MockMetadata,
    }

    impl TestOperations {
        pub fn new(metadata: MockMetadata) -> Self {
            Self { metadata }
        }

        pub fn metadata(&self) -> &MockMetadata {
            &self.metadata
        }

    //     /// Encodes a strongly-typed native slice directly into a TypedArray container variant
    //     /// and serializes it to raw uncompressed bytes using the Fix 2 streamlined layout.
    //     pub fn encode<T>(
    //         &self,
    //         input: &[Option<T>],
    //         fallback_no_data: T,
    //     ) -> OperationsResult<Vec<u8>>
    //     where
    //         T: ToTypedArray,
    //     {
    //         // 1. Convert optional native primitives to a target TypedArray variant
    //         let typed_array = T::to_typed_array_from_options(input, fallback_no_data);

    //         // 2. Delegate directly to the zero-copy utility encoder
    //         encode_typed_array(&typed_array)
    //     }
    // }

    }

    #[test]
    fn test_to_typed_array_variant_wrapping() {
        // Arrange standard optional sequences
        let input_data = vec![Some(5i16), None, Some(-20i16), Some(150i16)];
        let fallback_val = -999i16;

        // Act: Generate the dynamic TypedArray container block
        let result_typed_array = i16::to_typed_array_from_options(&input_data, fallback_val);

        // Assert: Ensure it matches the correct enum variant type and flattens correctly
        if let TypedArray::Int16(inner_vector) = result_typed_array {
            assert_eq!(inner_vector, vec![5, -999, -20, 150]);
        } else {
            panic!(
                "Expected resulting variant envelope mapping context to resolve to TypedArray::Int16"
            );
        }
    }

    // #[test]
    // fn test_encode_pipeline_byte_level_roundtrip() {
    //     // 1. Arrange: Setup spatial bounds mimicking a single-band 2x2 geospatial tile structure
    //     let metadata = MockMetadata {
    //         samples: 1,
    //         tile_size: 2,
    //         data_type: crate::DataType::Int32,
    //     };
    //     let operations_pipeline = TestOperations::new(metadata);

    //     // Provide standard 4 element matrix containing an unassigned data block entry
    //     let native_input: Vec<Option<i32>> = vec![Some(450), Some(-1200), None, Some(99999)];
    //     let no_data_fallback = -1;

    //     // Elements expected following the mapping evaluation
    //     let expected_flat_elements = vec![450i32, -1200i32, -1i32, 99999i32];

    //     // 2. Act: Execute the complete generic compile-time path
    //     let raw_output_bytes = operations_pipeline
    //         .encode(&native_input, no_data_fallback)
    //         .expect("Failed to execute top-level generic encode operation path");

    //     // 3. Assert: Verify the resulting bitstream payload represents a binary-perfect matching layout
    //     let reference_cast_bytes =
    //         bytemuck::cast_slice::<i32, u8>(&expected_flat_elements).to_vec();
    //     assert_eq!(raw_output_bytes, reference_cast_bytes);
    // }

    // #[test]
    // fn test_encode_and_compress_with_array_struct_parity() {
    //     // Using structural properties or builders matching rastertile-rs/src/metadata/mod.rs
    //     let metadata = Metadata::new(
    //         1,                     // samples / band channels count
    //         2,                     // tile_size rows (e.g. 2x2 grid quadrant)
    //         DataType::Int16,       // physical array DataType entry token
    //         crate::compression::DecompressionFormat::None, // baseline format settings
    //     );
        
    //     // Pass the absolute system type smoothly without operational parameters breaking
    //     let ops = Operations::new(metadata);

    //     let input_native: Vec<Option<i16>> = vec![Some(50), None, Some(-10), Some(200)];
    //     let fallback_val: i16 = -999;
    //     let expected_flat = vec![50i16, -999i16, -10i16, 200i16];

    //     // 1. Verify Uncompressed Encoding using the exact Array initialization step
    //     let uncompressed_bytes = ops
    //         .encode(&input_native, fallback_val)
    //         .expect("Failed to execute native encode through Array struct wrapper");
    //     let reference_bytes = bytemuck::cast_slice::<i16, u8>(&expected_flat).to_vec();
    //     assert_eq!(uncompressed_bytes, reference_bytes);

    //     // 2. Verify Compressed Step Compatibility
    //     let compression_config =
    //         CompressConfiguration::Gzip(crate::compression::GzipConfig { level: 6 });
    //     let compressed_bytes = ops
    //         .compress(&input_native, fallback_val, &compression_config)
    //         .expect("Failed to compress bytes generated from Array struct layout");

    //     assert!(!compressed_bytes.is_empty());
    //     assert_eq!(compressed_bytes[0], 0x1f); // Gzip Header Part 1
    //     assert_eq!(compressed_bytes[1], 0x8b); // Gzip Header Part 2
    // }
}
