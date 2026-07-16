#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileStatistics {
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

#[cfg(test)]
mod tests {

    use crate::operations::utils::no_data;

    #[test]
    fn test_get_pixel() {
        let nan_str = "";
        let my_float = no_data(crate::DataType::UInt8, nan_str.to_string());
        // nan_str.
        // let my_float= match nan_str.parse::<i32>(){
        //     Ok(val) => Some(val),
        //     Err(e)=> None,

        // };
        println!("{:?}", my_float);
        // my_float.

        // Check if the result is NaN
        // assert!(my_float.is_nan());
    }
}
