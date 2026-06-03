mod table_providers;
mod udf;
mod utils;

pub use crate::utils::get_tokio_runtime;

use pyo3::exceptions::PyRuntimeWarning;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3::{intern, wrap_pymodule};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[pyfunction]
fn ___version() -> &'static str {
    VERSION
}

/// Raise RuntimeWarning for debug builds
#[pyfunction]
fn check_debug_build(py: Python) -> PyResult<()> {
    #[cfg(debug_assertions)]
    {
        let warnings_mod = py.import(intern!(py, "warnings"))?;
        let warning = PyRuntimeWarning::new_err(
            "datafusion_raquet has not been compiled in release mode. Performance will be degraded.",
        );
        let args = PyTuple::new(py, vec![warning])?;
        warnings_mod.call_method1(intern!(py, "warn"), args)?;
    }
    Ok(())
}
/// A Python module implemented in Rust.
#[pymodule]
fn _internal(py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    check_debug_build(py)?;
    m.add_wrapped(wrap_pyfunction!(___version))?;

    // let quadbin_mod = wrap_pymodule!(udf::quadbin::quadbin)(py);
    // m.add_submodule(quadbin_mod.bind(py))?;
    // py.import(intern!(py, "sys"))?
    //     .getattr(intern!(py, "modules"))?
    //     .set_item("raquet_df.quadbin", quadbin_mod)?;

    let raster_mod = wrap_pymodule!(udf::rastertile::rastertile)(py);
    m.add_submodule(raster_mod.bind(py))?;
    py.import(intern!(py, "sys"))?
        .getattr(intern!(py, "modules"))?
        .set_item("datafusion_raquet.rastertile", raster_mod)?;

    let table_providers_mod = wrap_pymodule!(table_providers::table_providers)(py);
    m.add_submodule(table_providers_mod.bind(py))?;
    py.import(intern!(py, "sys"))?
        .getattr(intern!(py, "modules"))?
        .set_item("datafusion_raquet.table_providers", table_providers_mod)?;

    Ok(())
}

// pub use crate::table_providers::

// impl_table_provider!(RaquetTable, PyRaquetTable, "RaquetTable");

// #[pyclass(name = "ReadRaquet", module = "raquet_datafusion._internal")]
// #[derive(Debug, Clone)]
// pub struct PyReadRaquet(ReadRaquet);

// #[pymethods]
// impl PyReadRaquet {
//     #[new]
//     fn new() -> Self {
//         let inner = ReadRaquet::new();

//         PyReadRaquet(inner)
//     }

//     fn __datafusion_table_function__<'py>(
//         &self,
//         py: Python<'py>,
//         session: Bound<PyAny>,
//     ) -> PyResult<Bound<'py, PyCapsule>> {
//         let name = cr"datafusion_table_function".into();

//         let codec = ffi_logical_codec_from_pycapsule(session)?;
//         let func = self.0.clone();

//         let provider = FFI_TableFunction::new_with_ffi_codec(Arc::new(func), None, codec);

//         PyCapsule::new(py, provider, Some(name))
//     }
// }

//     pub fn __datafusion_table_provider__<'py>(
//         &self,
//         py: Python<'py>,
//         session: Bound<PyAny>,
//     ) -> PyResult<Bound<'py, PyCapsule>> {
//         let name = cr"datafusion_table_provider".into();
//         let codec = ffi_logical_codec_from_pycapsule(session)?;
//         let provider =
//             FFI_TableProvider::new_with_ffi_codec(self.table.clone(), false, None, codec);

//         PyCapsule::new(py, provider, Some(name))
//     }
// }

// #[pyclass(
//     from_py_object,
//     name = "ReadRaquet",
//     module = "raquet_datafusion._internal",
//     subclass
// )]
// #[derive(Debug, Clone)]
// pub struct PyReadRaquet {}

// #[pymethods]
// impl PyReadRaquet {
//     #[staticmethod]
//     pub fn serialize_to_plan(
//         sql: &str,
//         ctx: PySessionContext,
//         py: Python,
//     ) -> PyDataFusionResult<PyPlan> {
//         PySubstraitSerializer::serialize_bytes(sql, ctx, py).and_then(|proto_bytes| {
//             let proto_bytes = proto_bytes.bind(py).cast::<PyBytes>().unwrap();
//             PySubstraitSerializer::deserialize_bytes(proto_bytes.as_bytes().to_vec(), py)
//         })
//     }
// }
//     #[new]
//     fn new() -> Self {
//         Self {}
//     }

//     fn __datafusion_table_function__<'py>(
//         &self,
//         py: Python<'py>,
//         session: Bound<PyAny>,
//     ) -> PyResult<Bound<'py, PyCapsule>> {
//         let name = cr"datafusion_table_function".into();

//         let func = self.clone();
//         let codec = ffi_logical_codec_from_pycapsule(session)?;

//         let provider = FFI_TableFunction::new_with_ffi_codec(Arc::new(func), None, codec);

//         PyCapsule::new(py, provider, Some(name))
//     }
// }

// impl TableFunctionImpl for ReadRaquet {
//     fn call(&self, exprs: &[Expr]) -> Result<Arc<dyn TableProvider>> {
//         let Some(Expr::Literal(ScalarValue::Utf8(Some(table_name)), _)) = exprs.first() else {
//             return plan_err!("read_raquet requires at least one string argument");
//         };

//         // self.

//         let (schema, batches) = read_raquet_batches(path)?;

//         let table = ReadRaquetTableProvider { schema, batches };
//         Ok(Arc::new(table))
//     }
// }

// #[pymodule]
// fn _internal(m: &Bound<'_, PyModule>) -> PyResult<()> {
//     m.add_class::<PyRaquetTable>()?;
//     m.add_class::<PyReadRaquet>()?;

//     Ok(())
// }
