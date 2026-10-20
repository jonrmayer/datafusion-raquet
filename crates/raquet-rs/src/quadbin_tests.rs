#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Array, Float64Array, Int32Array, UInt64Array, UInt8Array, StructArray, ListArray};
    use crate::QuadbinOrchestrator;
    // London coordinates for standard operational validation baseline targets
    const SAMPLE_LON: f64 = -0.1278;
    const SAMPLE_LAT: f64 = 51.5074;
    const SAMPLE_RES: u8 = 10;

    #[test]
    fn test_quadbin_from_lonlat_and_back_batch() {
        let lon_arr = Float64Array::from(vec![Some(SAMPLE_LON), None, Some(SAMPLE_LON)]);
        let lat_arr = Float64Array::from(vec![Some(SAMPLE_LAT), None, Some(SAMPLE_LAT)]);
        let res_arr = UInt8Array::from(vec![Some(SAMPLE_RES), None, Some(SAMPLE_RES)]);

        // 1. Convert Geographic points to Quadbin cell index array
        let quadbin_res = QuadbinOrchestrator::quadbin_from_lonlat_batch(&lon_arr, &lat_arr, &res_arr).unwrap();
        assert_eq!(quadbin_res.len(), 3);
        assert!(quadbin_res.is_null(1)); // Null propagation check
        assert!(quadbin_res.is_valid(0));
        
        let valid_cell = quadbin_res.value(0);

        // 2. Decode back into coordinate fields
        let coords_struct = QuadbinOrchestrator::quadbin_to_lonlat_batch(&quadbin_res).unwrap();
        assert_eq!(coords_struct.len(), 3);
        assert!(coords_struct.is_null(1));

        let lons = coords_struct.column(0).as_any().downcast_ref::<Float64Array>().unwrap();
        let lats = coords_struct.column(1).as_any().downcast_ref::<Float64Array>().unwrap();

        // Spatial resolution checks allow slight floating point drift boundaries
        assert!((lons.value(0) - SAMPLE_LON).abs() < 0.1);
        assert!((lats.value(0) - SAMPLE_LAT).abs() < 0.1);
    }

    #[test]
    fn test_quadbin_tile_conversions_batch() {
        let x_arr = Int32Array::from(vec![Some(511), None]);
        let y_arr = Int32Array::from(vec![Some(340), None]);
        let z_arr = Int32Array::from(vec![Some(10), None]);

        // Tile coordinates to Quadbin array
        let cells = QuadbinOrchestrator::quadbin_from_tile_batch(&x_arr, &y_arr, &z_arr).unwrap();
        assert_eq!(cells.len(), 2);
        assert!(cells.is_null(1));

        // Quadbin cell token back to tile configuration structure
        let tiles_struct = QuadbinOrchestrator::quadbin_to_tile_batch(&cells).unwrap();
        let xs = tiles_struct.column(0).as_any().downcast_ref::<Int32Array>().unwrap();
        let ys = tiles_struct.column(1).as_any().downcast_ref::<Int32Array>().unwrap();
        let zs = tiles_struct.column(2).as_any().downcast_ref::<Int32Array>().unwrap();

        assert_eq!(xs.value(0), 511);
        assert_eq!(ys.value(0), 340);
        assert_eq!(zs.value(0), 10);
    }

    #[test]
    fn test_quadbin_hierarchy_and_resolution_batch() {
        let lon_arr = Float64Array::from(vec![Some(SAMPLE_LON)]);
        let lat_arr = Float64Array::from(vec![Some(SAMPLE_LAT)]);
        let res_arr = UInt8Array::from(vec![Some(SAMPLE_RES)]);

        let cells = QuadbinOrchestrator::quadbin_from_lonlat_batch(&lon_arr, &lat_arr, &res_arr).unwrap();
        
        // Test resolution level evaluation extraction window
        let res_eval = QuadbinOrchestrator::quadbin_resolution_batch(&cells).unwrap();
        assert_eq!(res_eval.value(0), SAMPLE_RES as i32);

        // Test parent conversion step tracking
        let target_parent_res = UInt8Array::from(vec![Some(SAMPLE_RES - 1)]);
        let parents = QuadbinOrchestrator::quadbin_to_parent_batch(&cells, &target_parent_res).unwrap();
        let parent_res_eval = QuadbinOrchestrator::quadbin_resolution_batch(&parents).unwrap();
        assert_eq!(parent_res_eval.value(0), (SAMPLE_RES - 1) as i32);
    }

    #[test]
    fn test_quadbin_kring_list_generation_batch() {
        let lon_arr = Float64Array::from(vec![Some(SAMPLE_LON), None]);
        let lat_arr = Float64Array::from(vec![Some(SAMPLE_LAT), None]);
        let res_arr = UInt8Array::from(vec![Some(SAMPLE_RES), None]);
        let cells = QuadbinOrchestrator::quadbin_from_lonlat_batch(&lon_arr, &lat_arr, &res_arr).unwrap();

        let k_arr = Int32Array::from(vec![Some(1), None]);

        // Unroll variable proximity rings into Arrow List structures
        let list_array = QuadbinOrchestrator::quadbin_kring_batch(&cells, &k_arr).unwrap();
        assert_eq!(list_array.len(), 2);
        assert!(list_array.is_null(1));

        // Ensure the offset tracking boundary correctly handles length slicing profiles
        let offsets = list_array.value_offsets();
        assert_eq!(offsets[0], 0);
        assert!(offsets[1] > 0); // K-ring at distance 1 yields several contiguous cells
    }
}
