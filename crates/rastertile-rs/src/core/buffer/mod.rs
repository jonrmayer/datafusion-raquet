use core::fmt;

use crate::core::error::{OxiGdalError, Result};

use crate::core::types::{NoDataValue, RasterDataType};
/// A typed buffer for raster data
#[derive(Clone)]
pub struct RasterBuffer {
    /// The underlying bytes
    data: Vec<u8>,
    /// Width in pixels
    width: u64,
    /// Height in pixels
    height: u64,
    /// Data type
    data_type: RasterDataType,
    /// `NoData` value
    nodata: NoDataValue,
}

impl RasterBuffer {
    /// Creates a new raster buffer
    ///
    /// # Errors
    /// Returns an error if the data size doesn't match the dimensions and type
    pub fn new(
        data: Vec<u8>,
        width: u64,
        height: u64,
        data_type: RasterDataType,
        nodata: NoDataValue,
    ) -> Result<Self> {
        let expected_size = width * height * data_type.size_bytes() as u64;
        if data.len() as u64 != expected_size {
            return Err(OxiGdalError::InvalidParameter {
                parameter: "data",
                message: format!(
                    "Data size mismatch: expected {} bytes for {}x{} {:?}, got {}",
                    expected_size,
                    width,
                    height,
                    data_type,
                    data.len()
                ),
            });
        }

        Ok(Self {
            data,
            width,
            height,
            data_type,
            nodata,
        })
    }

    /// Creates a zero-filled buffer
    #[must_use]
    pub fn zeros(width: u64, height: u64, data_type: RasterDataType) -> Self {
        let size = (width * height * data_type.size_bytes() as u64) as usize;
        Self {
            data: vec![0u8; size],
            width,
            height,
            data_type,
            nodata: NoDataValue::None,
        }
    }

    /// Creates a buffer filled with the nodata value
    #[must_use]
    pub fn nodata_filled(
        width: u64,
        height: u64,
        data_type: RasterDataType,
        nodata: NoDataValue,
    ) -> Self {
        let mut buffer = Self::zeros(width, height, data_type);
        buffer.nodata = nodata;

        // Fill with nodata value if defined
        if let Some(value) = nodata.as_f64() {
            buffer.fill_value(value);
        }

        buffer
    }

    /// Fills the buffer with a constant value
    pub fn fill_value(&mut self, value: f64) {
        match self.data_type {
            RasterDataType::UInt8 => {
                let v = value as u8;
                self.data.fill(v);
            }
            RasterDataType::Int8 => {
                let v = value as i8;
                self.data.fill(v as u8);
            }
            RasterDataType::UInt16 => {
                let v = (value as u16).to_ne_bytes();
                for chunk in self.data.chunks_exact_mut(2) {
                    chunk.copy_from_slice(&v);
                }
            }
            RasterDataType::Int16 => {
                let v = (value as i16).to_ne_bytes();
                for chunk in self.data.chunks_exact_mut(2) {
                    chunk.copy_from_slice(&v);
                }
            }
            RasterDataType::UInt32 => {
                let v = (value as u32).to_ne_bytes();
                for chunk in self.data.chunks_exact_mut(4) {
                    chunk.copy_from_slice(&v);
                }
            }
            RasterDataType::Int32 => {
                let v = (value as i32).to_ne_bytes();
                for chunk in self.data.chunks_exact_mut(4) {
                    chunk.copy_from_slice(&v);
                }
            }
            RasterDataType::Float32 => {
                let v = (value as f32).to_ne_bytes();
                for chunk in self.data.chunks_exact_mut(4) {
                    chunk.copy_from_slice(&v);
                }
            }
            RasterDataType::Float64 => {
                let v = value.to_ne_bytes();
                for chunk in self.data.chunks_exact_mut(8) {
                    chunk.copy_from_slice(&v);
                }
            }
            RasterDataType::UInt64 => {
                let v = (value as u64).to_ne_bytes();
                for chunk in self.data.chunks_exact_mut(8) {
                    chunk.copy_from_slice(&v);
                }
            }
            RasterDataType::Int64 => {
                let v = (value as i64).to_ne_bytes();
                for chunk in self.data.chunks_exact_mut(8) {
                    chunk.copy_from_slice(&v);
                }
            } // RasterDataType::CFloat32 | RasterDataType::CFloat64 => {
              //     // Complex types: fill with (value, 0)
              //     // This is a simplified implementation
              // }
        }
    }

    /// Returns the width in pixels
    #[must_use]
    pub const fn width(&self) -> u64 {
        self.width
    }

    /// Returns the height in pixels
    #[must_use]
    pub const fn height(&self) -> u64 {
        self.height
    }

    /// Returns the data type
    #[must_use]
    pub const fn data_type(&self) -> RasterDataType {
        self.data_type
    }

    /// Returns the nodata value
    #[must_use]
    pub const fn nodata(&self) -> NoDataValue {
        self.nodata
    }

    /// Returns the total number of pixels
    #[must_use]
    pub const fn pixel_count(&self) -> u64 {
        self.width * self.height
    }

    /// Returns the raw bytes
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Returns mutable raw bytes
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Consumes the buffer and returns the raw bytes
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }

    /// Creates a buffer from typed vector data
    ///
    /// # Arguments
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    /// * `data` - Typed data (e.g., `Vec<f32>`, `Vec<u8>`)
    /// * `data_type` - The raster data type
    ///
    /// # Errors
    /// Returns an error if the data size doesn't match dimensions and type
    pub fn from_typed_vec<T: Copy + 'static>(
        width: usize,
        height: usize,
        data: Vec<T>,
        data_type: RasterDataType,
    ) -> Result<Self> {
        let expected_pixels = width * height;
        if data.len() != expected_pixels {
            return Err(OxiGdalError::InvalidParameter {
                parameter: "data",
                message: format!(
                    "Data length mismatch: expected {} pixels for {}x{}, got {}",
                    expected_pixels,
                    width,
                    height,
                    data.len()
                ),
            });
        }

        // Convert typed data to bytes
        let type_size = core::mem::size_of::<T>();
        let expected_type_size = data_type.size_bytes();
        if type_size != expected_type_size {
            return Err(OxiGdalError::InvalidParameter {
                parameter: "data_type",
                message: format!(
                    "Type size mismatch: provided type has {} bytes, {:?} expects {} bytes",
                    type_size, data_type, expected_type_size
                ),
            });
        }

        let byte_data: Vec<u8> = data
            .iter()
            .flat_map(|v| {
                // SAFETY: We're reading the bytes of a Copy type
                let ptr = v as *const T as *const u8;
                unsafe { core::slice::from_raw_parts(ptr, type_size) }.to_vec()
            })
            .collect();

        Self::new(
            byte_data,
            width as u64,
            height as u64,
            data_type,
            NoDataValue::None,
        )
    }

    /// Returns the buffer data as a typed slice
    ///
    /// # Type Parameters
    /// * `T` - The target type (must match the buffer's data type size)
    ///
    /// # Errors
    /// Returns an error if the type size doesn't match the data type
    pub fn as_slice<T: Copy + 'static>(&self) -> Result<&[T]> {
        let type_size = core::mem::size_of::<T>();
        let expected_size = self.data_type.size_bytes();

        if type_size != expected_size {
            return Err(OxiGdalError::InvalidParameter {
                parameter: "T",
                message: format!(
                    "Type size mismatch: requested type has {} bytes, buffer contains {:?} ({} bytes)",
                    type_size, self.data_type, expected_size
                ),
            });
        }

        let pixel_count = (self.width * self.height) as usize;
        // SAFETY: We've verified the type size matches, and the data is properly aligned
        // for the original type it was created with
        let slice =
            unsafe { core::slice::from_raw_parts(self.data.as_ptr() as *const T, pixel_count) };
        Ok(slice)
    }

    /// Returns the buffer data as a mutable typed slice
    ///
    /// # Type Parameters
    /// * `T` - The target type (must match the buffer's data type size)
    ///
    /// # Errors
    /// Returns an error if the type size doesn't match the data type
    pub fn as_slice_mut<T: Copy + 'static>(&mut self) -> Result<&mut [T]> {
        let type_size = core::mem::size_of::<T>();
        let expected_size = self.data_type.size_bytes();

        if type_size != expected_size {
            return Err(OxiGdalError::InvalidParameter {
                parameter: "T",
                message: format!(
                    "Type size mismatch: requested type has {} bytes, buffer contains {:?} ({} bytes)",
                    type_size, self.data_type, expected_size
                ),
            });
        }

        let pixel_count = (self.width * self.height) as usize;
        // SAFETY: We've verified the type size matches, and the data is properly aligned
        // for the original type it was created with
        let slice = unsafe {
            core::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut T, pixel_count)
        };
        Ok(slice)
    }

    /// Gets a pixel value as f64
    ///
    /// # Errors
    /// Returns an error if coordinates are out of bounds
    pub fn get_pixel(&self, x: u64, y: u64) -> Result<f64> {
        if x >= self.width || y >= self.height {
            return Err(OxiGdalError::OutOfBounds {
                message: format!(
                    "Pixel ({}, {}) out of bounds for {}x{} buffer",
                    x, y, self.width, self.height
                ),
            });
        }

        let pixel_size = self.data_type.size_bytes();
        let offset = (y * self.width + x) as usize * pixel_size;

        let value = match self.data_type {
            RasterDataType::UInt8 => f64::from(self.data[offset]),
            RasterDataType::Int8 => f64::from(self.data[offset] as i8),
            RasterDataType::UInt16 => {
                let bytes: [u8; 2] = self.data[offset..offset + 2].try_into().map_err(|_| {
                    OxiGdalError::Internal {
                        message: "Invalid slice length".to_string(),
                    }
                })?;
                f64::from(u16::from_ne_bytes(bytes))
            }
            RasterDataType::Int16 => {
                let bytes: [u8; 2] = self.data[offset..offset + 2].try_into().map_err(|_| {
                    OxiGdalError::Internal {
                        message: "Invalid slice length".to_string(),
                    }
                })?;
                f64::from(i16::from_ne_bytes(bytes))
            }
            RasterDataType::UInt32 => {
                let bytes: [u8; 4] = self.data[offset..offset + 4].try_into().map_err(|_| {
                    OxiGdalError::Internal {
                        message: "Invalid slice length".to_string(),
                    }
                })?;
                f64::from(u32::from_ne_bytes(bytes))
            }
            RasterDataType::Int32 => {
                let bytes: [u8; 4] = self.data[offset..offset + 4].try_into().map_err(|_| {
                    OxiGdalError::Internal {
                        message: "Invalid slice length".to_string(),
                    }
                })?;
                f64::from(i32::from_ne_bytes(bytes))
            }
            RasterDataType::Float32 => {
                let bytes: [u8; 4] = self.data[offset..offset + 4].try_into().map_err(|_| {
                    OxiGdalError::Internal {
                        message: "Invalid slice length".to_string(),
                    }
                })?;
                f64::from(f32::from_ne_bytes(bytes))
            }
            RasterDataType::Float64 => {
                let bytes: [u8; 8] = self.data[offset..offset + 8].try_into().map_err(|_| {
                    OxiGdalError::Internal {
                        message: "Invalid slice length".to_string(),
                    }
                })?;
                f64::from_ne_bytes(bytes)
            }
            RasterDataType::UInt64 => {
                let bytes: [u8; 8] = self.data[offset..offset + 8].try_into().map_err(|_| {
                    OxiGdalError::Internal {
                        message: "Invalid slice length".to_string(),
                    }
                })?;
                u64::from_ne_bytes(bytes) as f64
            }
            RasterDataType::Int64 => {
                let bytes: [u8; 8] = self.data[offset..offset + 8].try_into().map_err(|_| {
                    OxiGdalError::Internal {
                        message: "Invalid slice length".to_string(),
                    }
                })?;
                i64::from_ne_bytes(bytes) as f64
            } // RasterDataType::CFloat32 | RasterDataType::CFloat64 => {
              //     // Return only the real part for complex types
              //     let bytes: [u8; 4] = self.data[offset..offset + 4].try_into().map_err(|_| {
              //         OxiGdalError::Internal {
              //             message: "Invalid slice length".to_string(),
              //         }
              //     })?;
              //     f64::from(f32::from_ne_bytes(bytes))
              // }
        };

        Ok(value)
    }

    /// Sets a pixel value
    ///
    /// # Errors
    /// Returns an error if coordinates are out of bounds
    pub fn set_pixel(&mut self, x: u64, y: u64, value: f64) -> Result<()> {
        if x >= self.width || y >= self.height {
            return Err(OxiGdalError::OutOfBounds {
                message: format!(
                    "Pixel ({}, {}) out of bounds for {}x{} buffer",
                    x, y, self.width, self.height
                ),
            });
        }

        let pixel_size = self.data_type.size_bytes();
        let offset = (y * self.width + x) as usize * pixel_size;

        match self.data_type {
            RasterDataType::UInt8 => {
                self.data[offset] = value as u8;
            }
            RasterDataType::Int8 => {
                self.data[offset] = (value as i8) as u8;
            }
            RasterDataType::UInt16 => {
                let bytes = (value as u16).to_ne_bytes();
                self.data[offset..offset + 2].copy_from_slice(&bytes);
            }
            RasterDataType::Int16 => {
                let bytes = (value as i16).to_ne_bytes();
                self.data[offset..offset + 2].copy_from_slice(&bytes);
            }
            RasterDataType::UInt32 => {
                let bytes = (value as u32).to_ne_bytes();
                self.data[offset..offset + 4].copy_from_slice(&bytes);
            }
            RasterDataType::Int32 => {
                let bytes = (value as i32).to_ne_bytes();
                self.data[offset..offset + 4].copy_from_slice(&bytes);
            }
            RasterDataType::Float32 => {
                let bytes = (value as f32).to_ne_bytes();
                self.data[offset..offset + 4].copy_from_slice(&bytes);
            }
            RasterDataType::Float64 => {
                let bytes = value.to_ne_bytes();
                self.data[offset..offset + 8].copy_from_slice(&bytes);
            }
            RasterDataType::UInt64 => {
                let bytes = (value as u64).to_ne_bytes();
                self.data[offset..offset + 8].copy_from_slice(&bytes);
            }
            RasterDataType::Int64 => {
                let bytes = (value as i64).to_ne_bytes();
                self.data[offset..offset + 8].copy_from_slice(&bytes);
            } // RasterDataType::CFloat32 => {
              //     // Set only the real part
              //     let bytes = (value as f32).to_ne_bytes();
              //     self.data[offset..offset + 4].copy_from_slice(&bytes);
              // }
              // RasterDataType::CFloat64 => {
              //     // Set only the real part
              //     let bytes = value.to_ne_bytes();
              //     self.data[offset..offset + 8].copy_from_slice(&bytes);
              // }
        }

        Ok(())
    }

    /// Returns true if the given value equals the nodata value
    #[must_use]
    pub fn is_nodata(&self, value: f64) -> bool {
        match self.nodata.as_f64() {
            Some(nd) => {
                if nd.is_nan() && value.is_nan() {
                    true
                } else {
                    (nd - value).abs() < f64::EPSILON
                }
            }
            None => false,
        }
    }

    /// Converts the buffer to a different data type
    ///
    /// # Errors
    /// Returns an error if conversion fails
    pub fn convert_to(&self, target_type: RasterDataType) -> Result<Self> {
        if target_type == self.data_type {
            return Ok(self.clone());
        }

        let mut result = Self::zeros(self.width, self.height, target_type);
        result.nodata = self.nodata;

        for y in 0..self.height {
            for x in 0..self.width {
                let value = self.get_pixel(x, y)?;
                result.set_pixel(x, y, value)?;
            }
        }

        Ok(result)
    }

    /// Computes basic statistics
    pub fn compute_statistics(&self) -> Result<BufferStatistics> {
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        let mut valid_count = 0u64;

        for y in 0..self.height {
            for x in 0..self.width {
                let value = self.get_pixel(x, y)?;
                if !self.is_nodata(value) && value.is_finite() {
                    min = min.min(value);
                    max = max.max(value);
                    sum += value;
                    sum_sq += value * value;
                    valid_count += 1;
                }
            }
        }

        if valid_count == 0 {
            return Ok(BufferStatistics {
                min: f64::NAN,
                max: f64::NAN,
                mean: f64::NAN,
                std_dev: f64::NAN,
                valid_count: 0,
            });
        }

        let mean = sum / valid_count as f64;
        let variance = (sum_sq / valid_count as f64) - (mean * mean);
        let std_dev = variance.sqrt();

        Ok(BufferStatistics {
            min,
            max,
            mean,
            std_dev,
            valid_count,
        })
    }

    /// Computes basic statistics
    pub fn compute_values(&self) -> Result<Vec<Option<f32>>> {
        let mut result: Vec<Option<f32>> = vec![];

        for y in 0..self.height {
            for x in 0..self.width {
                let value = self.get_pixel(x, y)?;
                let val_f32 = value as f32;
                result.push(Some(val_f32));
            }
        }

        Ok(result)
    }
}

impl fmt::Debug for RasterBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RasterBuffer")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("data_type", &self.data_type)
            .field("nodata", &self.nodata)
            .field("bytes", &self.data.len())
            .finish()
    }
}

/// Statistics computed from a buffer
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BufferStatistics {
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Mean value
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Number of valid (non-nodata) pixels
    pub valid_count: u64,
}
