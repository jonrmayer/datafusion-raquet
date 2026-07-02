use datafusion_catalog::{TableFunctionArgs, TableFunctionImpl, TableProvider};
use datafusion_common::error::Result as DataFusionResult;
use datafusion_ffi::udtf::FFI_TableFunction;
use datafusion_python_util::ffi_logical_codec_from_pycapsule;
use datafusion_raquet::views::ReadRaquet;
use pyo3::prelude::*;
use pyo3::types::PyCapsule;
use pyo3::{Bound, PyAny, PyResult, Python, pyclass, pymethods};
use std::sync::Arc;


#[pyclass(
    from_py_object,
    name = "ReadRaquetView",
    module = "datafusion_raquet",
    subclass
)]
#[derive(Debug, Clone)]
pub struct PyReadRaquetView(Arc<ReadRaquet>);

#[pyo3::pymethods]
impl PyReadRaquetView {
    #[new]
    fn new() -> Self {
        PyReadRaquetView(Arc::new(ReadRaquet{}))
    }

    fn __datafusion_table_function__<'py>(
        &self,
        py: Python<'py>,
        session: Bound<PyAny>,
    ) -> PyResult<Bound<'py, PyCapsule>> {
        let name = cr"datafusion_table_function".into();
        let func = self.0.clone();
        let codec = ffi_logical_codec_from_pycapsule(session)?;
        let provider = FFI_TableFunction::new_with_ffi_codec(func, None, codec);

        PyCapsule::new(py, provider, Some(name))
    }
}



#[pymodule]
pub(crate) fn view_tables(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyReadRaquetView>()?;

    Ok(())
}


