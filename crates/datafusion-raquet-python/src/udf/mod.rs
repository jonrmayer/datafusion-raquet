pub(crate) mod rastertile;
pub(crate) mod quadbin;

#[macro_export]
macro_rules! impl_udf {
    ($base:ident, $py_name:ident, $python_name:literal) => {
        #[::pyo3::pyclass(module = "raquet_df", name = $python_name, frozen)]
        #[derive(Debug, Clone)]
        pub struct $py_name(::std::sync::Arc<$base>);

        #[::pyo3::pymethods]
        impl $py_name {
            #[new]
            fn new() -> Self {
                $py_name(::std::sync::Arc::new($base::new()))
            }

            fn __datafusion_scalar_udf__<'py>(
                &self,
                py: ::pyo3::Python<'py>,
            ) -> ::pyo3::PyResult<::pyo3::Bound<'py, ::pyo3::types::PyCapsule>> {
                 let name = cr"datafusion_scalar_udf".into();
                let udf = ::std::sync::Arc::new(
                    ::datafusion::logical_expr::ScalarUDF::new_from_shared_impl(self.0.clone()),
                );
                ::pyo3::types::PyCapsule::new(
                    py,
                    ::datafusion_ffi::udf::FFI_ScalarUDF::from(udf),
                    Some(name),
                )
            }
        }
    };
}

