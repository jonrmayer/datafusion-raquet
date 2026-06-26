
use std::sync::Arc;
mod read_raquet;
mod read_raquet_metadata;
mod util;

pub use read_raquet_metadata::ReadRaquetMetadata;
pub use read_raquet::ReadRaquet;


pub fn register(session_context: &datafusion::prelude::SessionContext) {
    session_context.register_udtf("read_raquet", Arc::new(ReadRaquet {}));
    session_context.register_udtf("read_raquet_metadata", Arc::new(ReadRaquetMetadata {}));
   
}