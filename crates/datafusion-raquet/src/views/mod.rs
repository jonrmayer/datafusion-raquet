use std::sync::Arc;
mod read_raquet;
mod read_raquet_at;
mod read_raquet_metadata;
mod util;

pub use read_raquet::ReadRaquet;
pub use read_raquet_at::ReadRaquetAt;
pub use read_raquet_metadata::ReadRaquetMetadata;

pub fn register(session_context: &datafusion::prelude::SessionContext) {
    session_context.register_udtf("read_raquet", Arc::new(ReadRaquet {}));
    session_context.register_udtf("read_raquet_at", Arc::new(ReadRaquetAt {}));
    session_context.register_udtf("read_raquet_metadata", Arc::new(ReadRaquetMetadata {}));
}
