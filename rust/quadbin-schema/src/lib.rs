pub mod error;

mod metadata;

mod datatype;

mod r#type;

pub use metadata::Metadata;

pub use r#type::QuadbinType;

pub use datatype::QuadbinArrowType;
