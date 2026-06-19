use datafusion_raquet::udf::raster::{
    DecodeTile, NativeTile, RaquetPixel, StatisticsTile, TestFromTile,
};
use pyo3::prelude::*;

use crate::impl_udf;

impl_udf!(TestFromTile, PyTestFromTile, "TestFromTile");
impl_udf!(DecodeTile, PyDecodeTile, "DecodeTile");
impl_udf!(NativeTile, PyNativeTile, "NativeTile");

impl_udf!(RaquetPixel, PyRaquetPixel, "RaquetPixel");

impl_udf!(StatisticsTile, PyStatisticsTile, "StatisticsTile");

#[pymodule]
pub(crate) fn rastertile(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyTestFromTile>()?;
    m.add_class::<PyDecodeTile>()?;
    m.add_class::<PyNativeTile>()?;
    m.add_class::<PyStatisticsTile>()?;
    m.add_class::<PyRaquetPixel>()?;
    Ok(())
}
