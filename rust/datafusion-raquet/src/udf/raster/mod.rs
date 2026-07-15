mod decode;
mod decompress;
mod native;

mod raquet_value;

mod raquet_pixel;
mod statistics;
#[cfg(any(test, debug_assertions))]
pub mod testing;

pub mod utils;

pub use decode::DecodeTile;

pub use decompress::DecompressTile;
pub use native::NativeTile;

pub use raquet_pixel::RaquetPixel;
pub use raquet_value::RaquetValue;
pub use statistics::StatisticsTile;

pub fn register(session_context: &datafusion::prelude::SessionContext) {
    session_context.register_udf(DecompressTile::default().into());
    session_context.register_udf(DecodeTile::default().into());

    session_context.register_udf(NativeTile::default().into());
    session_context.register_udf(StatisticsTile::default().into());

    session_context.register_udf(RaquetPixel::default().into());
    session_context.register_udf(RaquetValue::default().into());
}
