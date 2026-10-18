use quadbin_rs::{QuadBin, QuadBinResult, QuadBinError, Bbox, Tile, PixelCoord};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RaquetError {
    #[error("Coordinate validation boundary error: longitude {lon}, latitude {lat} outside limits")]
    InvalidCoordinates { lon: f64, lat: f64 },
    #[error("Quadbin indexing resolution {0} is invalid (must be between 0 and 26)")]
    InvalidResolution(i8),
    #[error("Buffer mismatch: expected identical slice sizes but found lengths {0} and {1}")]
    BufferSizeMismatch(usize, usize),
    #[error("Underlying spatial tracking index initialization failed: {0:?}")]
    IndexInitializationError(QuadBinError),
    #[error("Failed to decode cell index to spatial coordinates: {0:?}")]
    CoordinateDecodingError(QuadBinError),
    #[error("Hierarchical operational error: {0:?}")]
    HierarchyError(QuadBinError),
}

/// The single orchestrator managing cross-crate routing, indexing math,
/// hierarchical structures, and flat array transformations.
pub struct RaquetOrchestrator;

impl RaquetOrchestrator {
    // =====================================================================
    // 1. CORE SCALAR INDEX MAPPING & FACTORIES
    // =====================================================================

    /// Encodes a geographic coordinate into an unwrapped structured `QuadBin` instance.
    #[inline]
    pub fn point_to_cell(lon: f64, lat: f64, resolution: i8) -> Result<QuadBin, RaquetError> {
        if !(0..=26).contains(&resolution) {
            return Err(RaquetError::InvalidResolution(resolution));
        }
        if !(-180.0..=180.0).contains(&lon) || !(-90.0..=90.0).contains(&lat) {
            return Err(RaquetError::InvalidCoordinates { lon, lat });
        }

        QuadBin::from_lonlat(lon, lat, resolution)
            .map_err(RaquetError::IndexInitializationError)
    }

    /// Decodes a structured `QuadBin` cell index back into a geographic longitude/latitude tuple.
    #[inline]
    pub fn cell_to_point(cell: QuadBin) -> Result<(f64, f64), RaquetError> {
        let center = cell.to_lonlat()
            .map_err(RaquetError::CoordinateDecodingError)?;
        Ok((center.0, center.1))
    }

    /// Converts explicit tile space grid metrics (x, y, zoom) directly into a structured `QuadBin` instance.
    #[inline]
    pub fn tile_to_cell(x: u32, y: u32, z: i8) -> Result<QuadBin, RaquetError> {
        if !(0..=26).contains(&z) {
            return Err(RaquetError::InvalidResolution(z));
        }
        
        let tile_input = Tile { x, y, z:z as u8 };
        QuadBin::from_tile(tile_input)
            .map_err(RaquetError::IndexInitializationError)
    }

    /// Converts a structured `QuadBin` index back into an independent `Tile` reference.
    #[inline]
    pub fn cell_to_tile(cell: QuadBin) -> Tile {
        cell.to_tile().unwrap()
        
    }

    /// Instantly returns the specific zoom scale or resolution metadata encoded inside a `QuadBin` instance.
    #[inline]
    pub fn get_resolution(cell: QuadBin) -> i8 {
        cell.resolution().unwrap() as i8
    }

    // // =====================================================================
    // // 2. BOUNDING BOX & RASTER PIXEL TRANSLATIONS
    // // =====================================================================

    // /// Extracts the complete geographic `Bbox` boundaries for a given `QuadBin` cell.
    // #[inline]
    // pub fn cell_to_bbox(cell: QuadBin) -> Result<Bbox, RaquetError> {
    //     cell.
    //     cell.bbox()
    //         .map_err(RaquetError::CoordinateDecodingError)
    // }

    /// Computes the exact grid pixel coordinates (`PixelCoord`) for a longitude/latitude point.
    #[inline]
    pub fn point_to_pixel(
        lon: f64, 
        lat: f64, 
        resolution: i8, 
        tile_size: u32
    ) -> Result<PixelCoord, RaquetError> {
        Ok(quadbin_rs::lonlat_to_pixel(lon, lat, resolution, tile_size as i16))
            // .map_err(RaquetError::IndexInitializationError)
    }

    // =====================================================================
    // 3. HIERARCHICAL OPERATIONS (UNWRAPPED INSTANCE METHODS)
    // =====================================================================

    // /// Retrieves the ancestor cell at a lower requested resolution via unwrapped instance evaluation.
    // #[inline]
    // pub fn get_parent(cell: QuadBin, parent_resolution: i8) -> Result<QuadBin, RaquetError> {
    //     // Safe unwrap of the instance method result
    //     cell.parent(parent_resolution)
    //         .map_err(RaquetError::HierarchyError)
    // }

    // /// Resolves an array containing all child cells for a specified resolution level via unwrapped instance evaluation.
    // #[inline]
    // pub fn get_children(cell: QuadBin, children_resolution: i8) -> Result<Vec<QuadBin>, RaquetError> {
    //     // Safe unwrap of the instance method result
    //     cell.children(children_resolution).unwrap()
    //         .map_err(RaquetError::HierarchyError)
    // }

    // /// Calculates the immediate k-ring geographic neighborhood array surrounding a target cell via unwrapped instance evaluation.
    // #[inline]
    // pub fn get_kring(cell: QuadBin, ring_size: u32) -> Result<Vec<QuadBin>, RaquetError> {
    //     // Safe unwrap of the instance method result
    //     cell.kring(ring_size)
    //         .map_err(RaquetError::HierarchyError)
    // }

    // =====================================================================
    // 4. HIGH-PERFORMANCE VECTORIZED BATCH EXTRACTIONS
    // =====================================================================

    /// Vectorized utility mapping an input memory slice of raw 64-bit cell integers 
    /// down into contiguous float coordinate output arrays.
    pub fn batch_cells_to_lonlat(
        cells: &[u64], 
        lons: &mut [f64], 
        lats: &mut [f64]
    ) -> Result<(), RaquetError> {
        if cells.len() != lons.len() || cells.len() != lats.len() {
            return Err(RaquetError::BufferSizeMismatch(cells.len(), lons.len()));
        }

        for (i, &cell_val) in cells.iter().enumerate() {
            match QuadBin::from_cell(cell_val){
                Ok(cell) => match cell.to_lonlat() {
                    Ok(center) => {
                        lons[i] = center.0;
                        lats[i] = center.1;
                    }
                    Err(_) => {
                        lons[i] = f64::NAN;
                        lats[i] = f64::NAN;
                    }
                },
                Err(_) => {
                    lons[i] = f64::NAN;
                    lats[i] = f64::NAN;
                }
            }
        }
        Ok(())
    }

    /// Vectorized utility converting flat contiguous coordinate array blocks into raw 64-bit cell values.
    pub fn batch_lonlat_to_cells(
        lons: &[f64], 
        lats: &[f64], 
        resolution: i8, 
        out_cells: &mut [u64]
    ) -> Result<(), RaquetError> {
        if !(0..=26).contains(&resolution) {
            return Err(RaquetError::InvalidResolution(resolution));
        }
        if lons.len() != lats.len() || lons.len() != out_cells.len() {
            return Err(RaquetError::BufferSizeMismatch(lons.len(), out_cells.len()));
        }

        for i in 0..lons.len() {
            if !(-180.0..=180.0).contains(&lons[i]) || !(-90.0..=90.0).contains(&lats[i]) {
                return Err(RaquetError::InvalidCoordinates { lon: lons[i], lat: lats[i] });
            }
            
            match QuadBin::from_lonlat(lons[i], lats[i], resolution) {
                Ok(cell) => {
                    out_cells[i] = cell.cell();
                }
                Err(e) => {
                    return Err(RaquetError::IndexInitializationError(e));
                }
            }
        }
        Ok(())
    }
}

// =====================================================================
// AUTOMATED UNIT TESTS FOR HIGH-PERFORMANCE BATCH PIPELINES
// =====================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_lonlat_to_cells_success() {
        // Arrange: Sample coordinates for London and New York at zoom level 4
        let lons = vec![-0.1278, -74.0060];
        let lats = vec![51.5074, 40.7128];
        let resolution = 4i8;
        let mut out_cells = vec![0u64; lons.len()];

        // Act: Execute the vectorized array transformation pipeline
        let result = RaquetOrchestrator::batch_lonlat_to_cells(
            &lons, 
            &lats, 
            resolution, 
            &mut out_cells
        );

        // Assert: Ensure execution completed cleanly and populated real non-zero integers
        assert!(result.is_ok());
        assert_ne!(out_cells[0], 0);
        assert_ne!(out_cells[1], 0);

        // Double check individual cell metadata values unwrap correctly
        let cell_0 = QuadBin::from_cell(out_cells[0]).unwrap();
        let cell_1 = QuadBin::from_cell(out_cells[1]).unwrap();
        assert_eq!(cell_0.resolution().unwrap(), resolution as u8);
        assert_eq!(cell_1.resolution().unwrap(), resolution as u8);
    }

    #[test]
    fn test_batch_lonlat_to_cells_invalid_resolution() {
        let lons = vec![0.0];
        let lats = vec![0.0];
        let invalid_resolution = 32i8; // Out of valid index scope limits (0-26)
        let mut out_cells = vec![0u64; 1];

        let result = RaquetOrchestrator::batch_lonlat_to_cells(
            &lons, 
            &lats, 
            invalid_resolution, 
            &mut out_cells
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            RaquetError::InvalidResolution(res) => assert_eq!(res, invalid_resolution),
            _ => panic!("Expected InvalidResolution error matrix layout"),
        }
    }

    #[test]
    fn test_batch_lonlat_to_cells_out_of_bounds_coordinates() {
        let lons = vec![200.0]; // Exceeds valid longitude limit (+180.0)
        let lats = vec![45.0];
        let mut out_cells = vec![0u64; 1];

        let result = RaquetOrchestrator::batch_lonlat_to_cells(&lons, &lats, 10, &mut out_cells);

        assert!(result.is_err());
        match result.unwrap_err() {
            RaquetError::InvalidCoordinates { lon, lat } => {
                assert_eq!(lon, 200.0);
                assert_eq!(lat, 45.0);
            }
            _ => panic!("Expected InvalidCoordinates pattern validation match"),
        }
    }

    #[test]
    fn test_batch_cells_to_lonlat_success_and_fallback() {
        // Arrange: Prepare an array with one valid cell index and one corrupt/malformed cell payload
        let valid_cell_struct = QuadBin::from_lonlat(0.0, 0.0, 4i8).unwrap();
        let valid_raw_u64 =valid_cell_struct.cell();
        let corrupt_raw_u64 = 999_999_999_u64; // Highly likely an invalid or non-decodable raw bitmask

        let cells = vec![valid_raw_u64, corrupt_raw_u64];
        let mut lons = vec![0.0; cells.len()];
        let mut lats = vec![0.0; cells.len()];

        // Act: Execute the cell decoding transformation pipeline
        let result = RaquetOrchestrator::batch_cells_to_lonlat(&cells, &mut lons, &mut lats);

        // Assert: The batch processor itself must return Ok(()) because it recovers internally
        assert!(result.is_ok());

        // Index 0: Valid mapping should result in real numbers approximately near our initial zero midpoint
        assert!(lons[0].is_finite());
        assert!(lats[0].is_finite());
        assert!((lons[0] - 0.0).abs() < 1.0); 

        // Index 1: Corrupt tracking index must be handled safely without causing a register panic
        // It drops into the fallback pattern and outputs an unwrapped float placeholder instead
        assert!(lons[1].is_nan());
        assert!(lats[1].is_nan());
    }

    #[test]
    fn test_batch_pipeline_size_mismatch_protection() {
        let cells = vec![111_111, 222_222];
        let mut insufficient_lons = vec![0.0; 1]; // Length is shorter than the source cells array slice
        let mut lats = vec![0.0; 2];

        let result = RaquetOrchestrator::batch_cells_to_lonlat(
            &cells, 
            &mut insufficient_lons, 
            &mut lats
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            RaquetError::BufferSizeMismatch(expected, found) => {
                assert_eq!(expected, 2);
                assert_eq!(found, 1);
            }
            _ => panic!("Expected BufferSizeMismatch crash barrier check to fire cleanly"),
        }
    }
}

