pub mod error;

pub mod format;
pub mod metadata;
pub mod tables;
pub mod udf;
// pub mod views;

pub use format::source::RaquetSource;

pub use metadata::{
    RaquetMetadataReader, raquet_band_metadata, raquet_format_from_str, raquet_quadbin_metadata,
};

pub use tables::raquet::RaquetTable;
// pub use views::read_raquet_metadata;

// pub use udf::raster::{NativeTile, StatisticsTile};

pub fn register(session_context: &datafusion::prelude::SessionContext) {
    crate::udf::quadbin::register(session_context);
    crate::udf::raster::register(session_context);
    crate::udf::general::register(session_context);

    // crate::views::register(session_context);
}
