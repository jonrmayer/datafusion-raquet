use pyo3::prelude::*;
use datafusion_raquet::RaquetTable;


#[macro_export]
macro_rules! impl_table_provider {
    ($base:ident, $py_name:ident, $python_name:literal) => {
        #[::pyo3::pyclass(module = "datafusion_raquet", name = $python_name, frozen)]
        #[derive(Debug, Clone)]
        pub struct $py_name (::std::sync::Arc<$base>);
        #[::pyo3::pymethods]
        impl $py_name {
           
            #[new]
            fn new(path: &str) -> Self {
                let path = path.to_string();
                let table = crate::utils::get_tokio_runtime().block_on(async { $base::from_path(path).await });
                $py_name(::std::sync::Arc::new(table))
            }

            pub fn __datafusion_table_provider__<'py>(
                &self,
                py: ::pyo3::Python<'py>,
                session: Bound<::pyo3::types::PyAny>,
            ) -> ::pyo3::PyResult<::pyo3::Bound<'py, ::pyo3::types::PyCapsule>> {
                let name = cr"datafusion_table_provider".into();
                let codec = ::datafusion_python_util::ffi_logical_codec_from_pycapsule(session)?;
                let provider =
                    ::datafusion_ffi::table_provider::FFI_TableProvider::new_with_ffi_codec(
                        self.0.clone(),
                        false,
                        None,
                        codec,
                    );

                ::pyo3::types::PyCapsule::new(py, provider, Some(name))
            }
        }
    };
}
impl_table_provider!(RaquetTable, PyRaquetTable, "RaquetTable");


#[pymodule]
pub(crate) fn table_providers(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyRaquetTable>()?;
    
    Ok(())
}