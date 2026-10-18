use ndarray::{Array3, Zip};
// use ndarray::{ Order};


// Import the authentic variants from your crate dependency
use crate::operations::TypedArray; 

/// Extension trait to convert a rastertile-rs TypedArray directly to your Array3 format
pub trait IntoArray3Pipeline {
    fn into_native_source_tile(self, bands: usize, height: usize, width: usize) -> NativeSourceTile;
    fn into_u8_mask_array3(self, bands: usize, height: usize, width: usize) -> Array3<u8>;
}

impl IntoArray3Pipeline for TypedArray {
    fn into_native_source_tile(self, bands: usize, height: usize, width: usize) -> NativeSourceTile {
        let shape = (bands, height, width);

        match self {
            // TypedArray::UInt8()
            TypedArray::UInt8(arr) => {
                let reshaped = Array3::from_shape_vec(shape, arr).expect("Dimension size mismatch for U8 source");
                NativeSourceTile::U8(reshaped)
            }
            TypedArray::Int16(arr) => {
                let reshaped = Array3::from_shape_vec(shape, arr).expect("Dimension size mismatch for I16 source");
                NativeSourceTile::I16(reshaped)
            }
            TypedArray::Float32(arr) => {
                let reshaped = Array3::from_shape_vec(shape, arr).expect("Dimension size mismatch for F32 source");
                NativeSourceTile::F32(reshaped)
            }
            TypedArray::Float64(arr) => {
                let reshaped = Array3::from_shape_vec(shape, arr).expect("Dimension size mismatch for F64 source");
                NativeSourceTile::F64(reshaped)
            }
            // Add any other structural integer variants supported by your local package
            _ => panic!("Unsupported primitive array variant decoded from raw_array()"),
        }
    }

    fn into_u8_mask_array3(self, bands: usize, height: usize, width: usize) -> Array3<u8> {
        let shape = (bands, height, width);

        match self {
            TypedArray::UInt8(arr) => {
                Array3::from_shape_vec(shape, arr).expect("Dimension size mismatch for U8 mask")
            }
            TypedArray::UInt16(arr) => {
                let reshaped = Array3::from_shape_vec(shape, arr).expect("Dimension size mismatch for U16 mask");
                reshaped.mapv(|v| if v > 0 { 1 } else { 0 })
            }
            TypedArray::Int32(arr) => {
                let reshaped = Array3::from_shape_vec(shape, arr).expect("Dimension size mismatch for I32 mask");
                reshaped.mapv(|v| if v > 0 { 1 } else { 0 })
            }
            _ => panic!("Unsupported array primitive format passed to mask pipeline"),
        }
    }
}


/// Enum wrapping your raster inputs in their native, untouched primitive layout
pub enum NativeSourceTile {
    U8(Array3<u8>),
    I8(Array3<i8>),
    U16(Array3<u16>),
    I16(Array3<i16>),
    U32(Array3<u32>),
    I32(Array3<i32>),
    U64(Array3<u64>),
    I64(Array3<i64>),
    F32(Array3<f32>),
    F64(Array3<f64>),
}


/// Generic function to mask any native array in place without dynamic float allocation
pub fn apply_native_in_memory_mask<T>(data_tile: &mut Array3<T>, mask_tile: &Array3<u8>, nodata_value: T)
where
    T: Copy + PartialEq,
{
    // Ensure geographical bounding shapes line up perfectly across bands
    assert_eq!(data_tile.shape(), mask_tile.shape(), "Array dimensions must match!");

    // Execute zero-copy loop matching your layout schema 
    Zip::from(data_tile)
        .and(mask_tile)
        .for_each(|data_pixel, &mask_pixel| {
            if mask_pixel == 0 {
                *data_pixel = nodata_value; // Assign the relevant native type fallback flag
            }
        });
}


/// Dispatches the generic mask logic directly to whichever native primitive type is active
pub fn process_native_tile(mut source: NativeSourceTile, mask: &Array3<u8>) -> NativeSourceTile {
    match source {
        NativeSourceTile::U8(ref mut arr)  => apply_native_in_memory_mask(arr, mask, 0u8),
        NativeSourceTile::I8(ref mut arr)  => apply_native_in_memory_mask(arr, mask, -1i8),
        NativeSourceTile::U16(ref mut arr) => apply_native_in_memory_mask(arr, mask, 0u16),
        NativeSourceTile::I16(ref mut arr) => apply_native_in_memory_mask(arr, mask, -32768i16), // Standard DEM NoData
        NativeSourceTile::U32(ref mut arr) => apply_native_in_memory_mask(arr, mask, 0u32),
        NativeSourceTile::I32(ref mut arr) => apply_native_in_memory_mask(arr, mask, -9999i32),
        NativeSourceTile::U64(ref mut arr) => apply_native_in_memory_mask(arr, mask, 0u64),
        NativeSourceTile::I64(ref mut arr) => apply_native_in_memory_mask(arr, mask, -9999i64),
        NativeSourceTile::F32(ref mut arr) => apply_native_in_memory_mask(arr, mask, f32::NAN),
        NativeSourceTile::F64(ref mut arr) => apply_native_in_memory_mask(arr, mask, f64::NAN),
    };
    source
}

 /// Unified execution method to decode and apply masks entirely natively
pub fn process_raw_tiles(
    raw_source_array: TypedArray,
    raw_mask_array: TypedArray,
    bands: usize,
    height: usize,
    width: usize,
) -> NativeSourceTile {
    // 1. Structural conversion from TypedArray straight into standard Array3 containers
    let native_source = raw_source_array.into_native_source_tile(bands, height, width);
    let normalized_mask = raw_mask_array.into_u8_mask_array3(bands, height, width);

    // 2. Dispatch down to your in-place native processing logic
    process_native_tile(native_source, &normalized_mask)
}
#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array3;

    #[test]
    fn test_process_native_tile_f32() {
        // Create a 1x2x2 (Bands, Height, Width) source tile filled with 42.0
        let source_arr = Array3::from_elem((1, 2, 2), 42.0f32);
        let source_tile = NativeSourceTile::F32(source_arr);

        // Create a mask where index [0, 0, 1] is invalid (0)
        let mut mask_arr = Array3::from_elem((1, 2, 2), 1u8);
        mask_arr[[0, 0, 1]] = 0; // Mark this pixel as invalid

        // Run the native routing pipeline
        let result_tile = process_native_tile(source_tile, &mask_arr);

        // Extract and verify results
        if let NativeSourceTile::F32(output_arr) = result_tile {
            // Valid pixel remains untouched
            assert_eq!(output_arr[[0, 0, 0]], 42.0);
            // Invalid pixel must match f32::NAN
            assert!(output_arr[[0, 0, 1]].is_nan());
            // Valid pixel remains untouched
            assert_eq!(output_arr[[0, 1, 0]], 42.0);
        } else {
            panic!("Expected NativeSourceTile::F32 variant missing.");
        }
    }

    #[test]
    fn test_process_native_tile_i16() {
        // Create a 1x2x2 source tile using signed 16-bit integers (e.g. elevation DEM)
        let source_arr = Array3::from_elem((1, 2, 2), 1500i16);
        let source_tile = NativeSourceTile::I16(source_arr);

        // Create a mask where index [0, 1, 1] is invalid (0)
        let mut mask_arr = Array3::from_elem((1, 2, 2), 1u8);
        mask_arr[[0, 1, 1]] = 0; 

        // Run the pipeline
        let result_tile = process_native_tile(source_tile, &mask_arr);

        // Extract and verify integer sentinel bounds
        if let NativeSourceTile::I16(output_arr) = result_tile {
            assert_eq!(output_arr[[0, 0, 0]], 1500);
            // Invalid integer pixel must match the configured NoData value (-32768)
            assert_eq!(output_arr[[0, 1, 1]], -32768);
        } else {
            panic!("Expected NativeSourceTile::I16 variant missing.");
        }
    }

    #[test]
    #[should_panic(expected = "Array dimensions must match!")]
    fn test_process_native_tile_dimension_mismatch_panics() {
        // Source tile shape is (1, 2, 2)
        let source_arr = Array3::from_elem((1, 2, 2), 10u8);
        let source_tile = NativeSourceTile::U8(source_arr);

        // Mask tile has mismatched shape (1, 3, 3)
        let mismatched_mask = Array3::from_elem((1, 3, 3), 1u8);

        // This operation must trigger the internal assert_eq! barrier safely
        let _ = process_native_tile(source_tile, &mismatched_mask);
    }
}
#[cfg(test)]
mod final_integration_tests {
    use super::*;
    use ndarray::Array3;
    // use rastertile_rs::TypedArray;

    #[test]
    fn test_process_raw_tiles_f32_with_raw_vec() {
        // 1. Dimensions setup (1 band, 2x2 grid = 4 pixels)
        let bands = 1;
        let height = 2;
        let width = 2;

        // 2. Feed a raw flat vector directly into the Float32 initializer
        let mock_source_data = vec![10.0f32, 20.0f32, 30.0f32, 40.0f32];
        let mock_raw_source = TypedArray::Float32(mock_source_data);

        // 3. Feed a raw mask vector directly into the UInt8 initializer
        let mock_mask_data = vec![1u8, 0u8, 1u8, 1u8]; // Index [0, 0, 1] is masked out
        let mock_raw_mask = TypedArray::UInt8(mock_mask_data);

        // 4. Run the pipeline logic
        let result_tile = process_raw_tiles(mock_raw_source, mock_raw_mask, bands, height, width);

        // 5. Evaluate results
        if let NativeSourceTile::F32(output_arr) = result_tile {
            assert_eq!(output_arr[[0, 0, 0]], 10.0);
            assert_eq!(output_arr[[0, 1, 0]], 30.0);
            assert_eq!(output_arr[[0, 1, 1]], 40.0);

            // Validates that the index evaluated as 0 correctly assigned an f32::NAN
            assert!(output_arr[[0, 0, 1]].is_nan());
        } else {
            panic!("Expected NativeSourceTile::F32 variant missing.");
        }
    }

    #[test]
    fn test_process_raw_tiles_i16_with_raw_vec() {
        let bands = 1;
        let height = 2;
        let width = 2;

        let mock_source_data = vec![100i16, 200i16, 300i16, 400i16];
        let mock_raw_source = TypedArray::Int16(mock_source_data);

        let mock_mask_data = vec![1u8, 1u8, 1u8, 0u8]; // Mask out the last pixel
        let mock_raw_mask = TypedArray::UInt8(mock_mask_data);

        let result_tile = process_raw_tiles(mock_raw_source, mock_raw_mask, bands, height, width);

        if let NativeSourceTile::I16(output_arr) = result_tile {
            assert_eq!(output_arr[[0, 0, 0]], 100);
            // Integer pixel must be overwritten with the native NoData sentinel layout (-32768)
            assert_eq!(output_arr[[0, 1, 1]], -32768);
        } else {
            panic!("Expected NativeSourceTile::I16 variant missing.");
        }
    }
}
