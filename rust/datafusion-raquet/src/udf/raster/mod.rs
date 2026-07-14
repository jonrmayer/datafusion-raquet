mod decode;
mod decompress;
mod native;

mod parquet_decode;

mod parquet_value;
mod raquet_value;


mod parquet_statistics;
mod raquet_pixel;
mod statistics;
#[cfg(any(test, debug_assertions))]
pub mod testing;

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
    
}
