use pyo3::prelude::*;

use datafusion_raquet::udf::quadbin::{
    QuadBinFromLonLat, QuadBinFromTile, QuadBinKRing, QuadBinResolution, QuadBinToBBOX,
    QuadBinToBBOXMercator, QuadBinToBBOXWGS84, QuadBinToChildren, QuadBinToGeoJSON,
    QuadBinToLonLat, QuadBinToParent, QuadBinToSibling, QuadBinToTile, QuadBinToWKT,
};

use crate::impl_udf;

impl_udf!(QuadBinFromTile, PyQuadBinFromTile, "QuadBinFromTile");

impl_udf!(QuadBinToTile, PyQuadBinToTile, "QuadBinToTile");
impl_udf!(QuadBinFromLonLat, PyQuadBinFromLonLat, "QuadBinFromLonLat");
impl_udf!(QuadBinToParent, PyQuadBinToParent, "QuadBinToParent");
impl_udf!(QuadBinResolution, PyQuadBinResolution, "QuadBinResolution");
impl_udf!(QuadBinToChildren, PyQuadBinToChildren, "QuadBinToChildren");
impl_udf!(QuadBinToSibling, PyQuadBinToSibling, "QuadBinToSibling");
impl_udf!(QuadBinKRing, PyQuadBinKRing, "QuadBinKRing");

impl_udf!(QuadBinToBBOX, PyQuadBinToBBOX, "QuadBinToBBOX");

impl_udf!(
    QuadBinToBBOXMercator,
    PyQuadBinToBBOXMercator,
    "QuadBinToBBOXMercator"
);
impl_udf!(
    QuadBinToBBOXWGS84,
    PyQuadBinToBBOXWGS84,
    "QuadBinToBBOXWGS84"
);
impl_udf!(QuadBinToLonLat, PyQuadBinToLonLat, "QuadBinToLonLat");
// impl_udf!(QuadBinToPixelXY, PyQuadBinToPixelXY, "QuadBinToPixelXY");
impl_udf!(QuadBinToWKT, PyQuadBinToWKT, "QuadBinToWKT");
impl_udf!(QuadBinToGeoJSON, PyQuadBinToGeoJSON, "QuadBinToGeoJSON");

#[pymodule]
pub(crate) fn quadbin(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyQuadBinFromTile>()?;
    m.add_class::<PyQuadBinToTile>()?;
    m.add_class::<PyQuadBinFromLonLat>()?;
    m.add_class::<PyQuadBinToParent>()?;
    m.add_class::<PyQuadBinResolution>()?;
    m.add_class::<PyQuadBinToChildren>()?;
    m.add_class::<PyQuadBinToSibling>()?;
    m.add_class::<PyQuadBinKRing>()?;

    m.add_class::<PyQuadBinToBBOX>()?;
    m.add_class::<PyQuadBinToBBOXMercator>()?;
    m.add_class::<PyQuadBinToBBOXWGS84>()?;
    m.add_class::<PyQuadBinToLonLat>()?;
    // m.add_class::<PyQuadBinToPixelXY>()?;
    m.add_class::<PyQuadBinToWKT>()?;
    m.add_class::<PyQuadBinToGeoJSON>()?;

    Ok(())
}
