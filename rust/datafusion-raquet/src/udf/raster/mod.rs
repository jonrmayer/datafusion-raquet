// mod native_builders;
// pub use native_builders::{
// convert_list_array_i8,
// convert_list_array_f32
// };

mod decode;
mod decompress;
mod native;

mod raquet_value;

mod raquet_pixel;
mod statistics;
// mod test;

// mod utils;

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
