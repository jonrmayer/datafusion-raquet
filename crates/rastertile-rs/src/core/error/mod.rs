pub mod types;
pub use types::*;

pub type Result<T> = core::result::Result<T, OxiGdalError>;