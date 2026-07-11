pub mod intersects;
pub mod band_metadata;
pub mod quadbin_metadata;

pub use band_metadata::BandMetadata;
pub use intersects::Intersects;
pub use quadbin_metadata::QuadbinMetadata;

pub fn register(session_context: &datafusion::prelude::SessionContext) {
    session_context.register_udf(BandMetadata::default().into());
    session_context.register_udf(Intersects::default().into());
    session_context.register_udf(QuadbinMetadata::default().into());

}
