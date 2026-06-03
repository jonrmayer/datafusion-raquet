mod decode;
mod test;
mod native;
mod statistics;

pub use test::TestFromTile;
pub use decode::DecodeTile;
pub use native::NativeTile;
pub use statistics::StatisticsTile;
pub fn register(session_context: &datafusion::prelude::SessionContext) {
    session_context.register_udf(TestFromTile::default().into());
    session_context.register_udf(DecodeTile::default().into());
    session_context.register_udf(NativeTile::default().into());
    session_context.register_udf(StatisticsTile::default().into());

    // session_context.register_udf(QuadBinToBBOXMercator::default().into());
    // session_context.register_udf(QuadBinToBBOXWGS84::default().into());
    // session_context.register_udf(QuadBinToPixelXY::default().into());
    // session_context.register_udf(QuadBinToWKT::default().into());
    // session_context.register_udf(QuadBinToGeoJSON::default().into());
}
