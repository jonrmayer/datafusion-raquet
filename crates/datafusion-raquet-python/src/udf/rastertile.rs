use pyo3::prelude::*;
use datafusion_raquet::udf::raster::{DecodeTile, NativeTile, StatisticsTile, TestFromTile};


use crate::impl_udf;

impl_udf!(TestFromTile, PyTestFromTile, "TestFromTile");
impl_udf!(DecodeTile, PyDecodeTile, "DecodeTile");
impl_udf!(NativeTile, PyNativeTile, "NativeTile");

impl_udf!(StatisticsTile, PyStatisticsTile, "StatisticsTile");


#[pymodule]
pub(crate) fn rastertile(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyTestFromTile>()?;
    m.add_class::<PyDecodeTile>()?;
    m.add_class::<PyNativeTile>()?;
    m.add_class::<PyStatisticsTile>()?;
    Ok(())
}
