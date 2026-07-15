mod band_metadata;
mod intersects;
mod quadbin_metadata;
mod quadbin_pixel_xy;
mod quadbin_polyfill;
mod binary_to_raquet;

#[cfg(any(test, debug_assertions))]
pub mod testing;



pub use band_metadata::BandMetadata;
pub use intersects::Intersects;
pub use quadbin_metadata::QuadbinMetadata;
pub use quadbin_pixel_xy::QuadBinToPixelXY;
pub use quadbin_polyfill::QuadBinPolyFill;
pub use binary_to_raquet::CastRaquet;

pub fn register(session_context: &datafusion::prelude::SessionContext) {
    session_context.register_udf(BandMetadata::default().into());
    session_context.register_udf(Intersects::default().into());
    session_context.register_udf(QuadbinMetadata::default().into());
    session_context.register_udf(QuadBinToPixelXY::default().into());
    session_context.register_udf(QuadBinPolyFill::default().into());
    session_context.register_udf(CastRaquet::default().into());
}
