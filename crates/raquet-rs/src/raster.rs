use crate::error::Result;
use rastertile_rs::{
    Operations,
    BandMetadata, DataType, CompressionType, BandStatistics, array::NativeType
};

/// `RaquetOrchestrator` abstracts cross-crate calls directly through the central
/// unified `rastertile_rs::operations::Operations` dispatcher ecosystem.
pub struct RaquetOrchestrator;

impl RaquetOrchestrator {
    
    // ==========================================
    // Decompression & Decoding Operations
    // ==========================================

    /// Abstracts the call via `Operations` to decompress raw tile byte streams.
    #[inline]
    pub fn decompress(band_bytes: &[u8], compression: &CompressionType) -> Result<Vec<u8>> {
        Operations::decompress_bytes(band_bytes, compression)
            .map_err(|e| e.into())
    }

    /// Abstracts the call via `Operations` to unpack tile arrays into flat native primitive vectors.
    #[inline]
    pub fn decode_band_native<T>(band_bytes: &[u8], metadata: &BandMetadata) -> Result<Vec<T>>
    where
        T: NativeType + Default + Clone,
    {
        let decompressed = Self::decompress(band_bytes, &metadata.compression)?;
        Operations::decode_buffer::<T>(&decompressed, &metadata.data_type)
            .map_err(|e| e.into())
    }

    /// Abstracts the call via `Operations` to pack a primitive data stream back into a compressed Raquet BLOB.
    pub fn binary_to_raquet(
        raw_buffer: &[u8], 
        tile_size: u32, 
        binary_type: &str, 
        data_type: DataType, 
        no_data: Option<f64>, 
        compression: CompressionType
    ) -> Result<Vec<u8>> {
        let metadata = BandMetadata {
            tile_size,
            binary_type: binary_type.to_string(),
            data_type,
            no_data,
            compression,
        };
        Operations::pack_to_raquet_blob(raw_buffer, &metadata)
            .map_err(|e| e.into())
    }

    // ==========================================
    // Summary Statistics Operations
    // ==========================================

    /// Abstracts the call via `Operations` to calculate tile summaries.
    pub fn compute_statistics(band_bytes: &[u8], metadata: &BandMetadata) -> Result<BandStatistics> {
        let decompressed = Self::decompress(band_bytes, &metadata.compression)?;
        Operations::compute_buffer_stats(&decompressed, metadata)
            .map_err(|e| e.into())
    }

    // ==========================================
    // Spatial Array & Pixel Lookups
    // ==========================================

    /// Abstracts the call via `Operations` for relative pixel coordinate matrix queries.
    #[inline]
    pub fn get_pixel_value(band_bytes: &[u8], metadata: &BandMetadata, pixel_x: u32, pixel_y: u32) -> Result<f64> {
        let decompressed = Self::decompress(band_bytes, &metadata.compression)?;
        Operations::extract_pixel_by_xy(&decompressed, metadata, pixel_x, pixel_y)
            .map_err(|e| e.into())
    }

    /// Abstracts the call via `Operations` for geometric point WKT resolutions.
    #[inline]
    pub fn get_value_by_wkt(band_bytes: &[u8], metadata: &BandMetadata, wkt_point: &str, resolution: u8) -> Result<f64> {
        let decompressed = Self::decompress(band_bytes, &metadata.compression)?;
        Operations::extract_value_by_wkt(&decompressed, metadata, wkt_point, resolution)
            .map_err(|e| e.into())
    }

    // ==========================================
    // Clipping & Masking Operations
    // ==========================================

    /// Abstracts structural pipeline execution using `Operations` for clipping, masking, and re-compression.
    /// 
    /// This routes the step-by-step transformations strictly through the unified Operations interface:
    /// 1. Unpacking the binary stream from compressed format.
    /// 2. Executing mask calculations against geospatial geometries.
    /// 3. Re-compressing the post-mask buffer array back into the Raquet specification wire format.
    pub fn clip_and_mask(
        band_bytes: &[u8],
        metadata: &BandMetadata,
        geometry_wkt: &str,
        invert_mask: bool,
    ) -> Result<Vec<u8>> {
        let decompressed = Self::decompress(band_bytes, &metadata.compression)?;
        
        let processed_buffer = Operations::apply_geometry_mask(&decompressed, metadata, geometry_wkt, invert_mask)
            .map_err(|e| e.into())?;
            
        Operations::compress_bytes(&processed_buffer, &metadata.compression)
            .map_err(|e| e.into())
    }
}
