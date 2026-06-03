use datafusion_ffi::table_provider::FFI_TableProvider;
use datafusion_python_util::ffi_logical_codec_from_pycapsule;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyCapsule;
use std::sync::Arc;



pub fn get_tokio_runtime() -> &'static tokio::runtime::Runtime {
    use std::sync::OnceLock;
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime"))
}

// #[macro_export]
// macro_rules! impl_table_provider {
//     ($base:ident, $py_name:ident, $python_name:literal) => {
//         #[::pyo3::pyclass(module = "raquet_datafusion._internal", name = $python_name, frozen)]
//         #[derive(Debug, Clone)]
//         pub struct $py_name (::std::sync::Arc<$base>);
//         #[::pyo3::pymethods]
//         impl $py_name {
           
//             #[new]
//             fn new(path: &str) -> Self {
//                 let path = path.to_string();
//                 let table = utils::get_tokio_runtime().block_on(async { $base::from_path(path).await });
//                 $py_name(::std::sync::Arc::new(table))
//             }

//             pub fn __datafusion_table_provider__<'py>(
//                 &self,
//                 py: ::pyo3::Python<'py>,
//                 session: Bound<::pyo3::types::PyAny>,
//             ) -> ::pyo3::PyResult<::pyo3::Bound<'py, ::pyo3::types::PyCapsule>> {
//                 let name = cr"datafusion_table_provider".into();
//                 let codec = ::datafusion_python_util::ffi_logical_codec_from_pycapsule(session)?;
//                 let provider =
//                     ::datafusion_ffi::table_provider::FFI_TableProvider::new_with_ffi_codec(
//                         self.0.clone(),
//                         false,
//                         None,
//                         codec,
//                     );

//                 ::pyo3::types::PyCapsule::new(py, provider, Some(name))
//             }
//         }
//     };
// }


// #[macro_export]
// macro_rules! impl_table_function {
//     ($base:ident, $py_name:ident, $python_name:literal) => {
//         #[::pyo3::pyclass(module = "raquet_datafusion._internal", name = $python_name, frozen)]
//         #[derive(Debug, Clone)]
//         pub struct $py_name (::std::sync::Arc<$base>);
//         #[::pyo3::pymethods]
//         impl $py_name {
           
//             #[new]
//             fn new(path: &str) -> Self {
//                 let path = path.to_string();
//                 let table = utils::get_tokio_runtime().block_on(async { $base::from_path(path).await });
//                 $py_name(::std::sync::Arc::new(table))
//             }

//             pub fn __datafusion_table_provider__<'py>(
//                 &self,
//                 py: ::pyo3::Python<'py>,
//                 session: Bound<::pyo3::types::PyAny>,
//             ) -> ::pyo3::PyResult<::pyo3::Bound<'py, ::pyo3::types::PyCapsule>> {
//                 let name = cr"datafusion_table_provider".into();
//                 let codec = ::datafusion_python_util::ffi_logical_codec_from_pycapsule(session)?;
//                 let provider =
//                     ::datafusion_ffi::table_provider::FFI_TableProvider::new_with_ffi_codec(
//                         self.0.clone(),
//                         false,
//                         None,
//                         codec,
//                     );

//                 ::pyo3::types::PyCapsule::new(py, provider, Some(name))
//             }
//         }
//     };
// }

// /// Simple macro to generate Python wrappers for geodatafusion UDF structs
// /// Usage: impl_udf!(RustStructName, PyWrapperName, "python_name")
// #[macro_export]
// macro_rules! impl_udf {
//     ($base:ident, $py_name:ident, $python_name:literal) => {
//         #[::pyo3::pyclass(module = "raquet_df", name = $python_name, frozen)]
//         #[derive(Debug, Clone)]
//         pub struct $py_name(::std::sync::Arc<$base>);

//         #[::pyo3::pymethods]
//         impl $py_name {
//             #[new]
//             fn new() -> Self {
//                 $py_name(::std::sync::Arc::new($base::new()))
//             }

//             fn __datafusion_scalar_udf__<'py>(
//                 &self,
//                 py: ::pyo3::Python<'py>,
//             ) -> ::pyo3::PyResult<::pyo3::Bound<'py, ::pyo3::types::PyCapsule>> {
//                 let udf = ::std::sync::Arc::new(
//                     ::datafusion::logical_expr::ScalarUDF::new_from_shared_impl(self.0.clone()),
//                 );
//                 ::pyo3::types::PyCapsule::new(
//                     py,
//                     ::datafusion_ffi::udf::FFI_ScalarUDF::from(udf),
//                     Some($crate::constants::SCALAR_UDF_CAPSULE_NAME.into()),
//                 )
//             }
//         }
//     };
// }
