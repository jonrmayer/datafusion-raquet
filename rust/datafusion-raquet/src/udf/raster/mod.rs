// mod native_builders;
// pub use native_builders::{
// convert_list_array_i8,
// convert_list_array_f32
// };

mod decode;
mod decompress;
mod native;
// mod parquet_native;
mod parquet_decode;

mod parquet_value;
mod raquet_value;

mod cast_raquet;
mod parquet_statistics;
mod raquet_pixel;
mod statistics;
// mod test;

mod utils;

pub use decode::DecodeTile;
pub use parquet_decode::ParquetDecodeTile;

pub use decompress::DecompressTile;
pub use native::NativeTile;
pub use parquet_statistics::ParquetStatisticsTile;
pub use parquet_value::ParquetValue;
pub use raquet_pixel::RaquetPixel;
pub use raquet_value::RaquetValue;
pub use statistics::StatisticsTile;

pub use cast_raquet::CastRaquet;

pub fn register(session_context: &datafusion::prelude::SessionContext) {
    session_context.register_udf(DecompressTile::default().into());
    session_context.register_udf(DecodeTile::default().into());
    session_context.register_udf(ParquetDecodeTile::default().into());
    session_context.register_udf(NativeTile::default().into());
    session_context.register_udf(StatisticsTile::default().into());
    session_context.register_udf(ParquetStatisticsTile::default().into());
    session_context.register_udf(RaquetPixel::default().into());
    session_context.register_udf(RaquetValue::default().into());
    session_context.register_udf(ParquetValue::default().into());
    session_context.register_udf(CastRaquet::default().into());
}
