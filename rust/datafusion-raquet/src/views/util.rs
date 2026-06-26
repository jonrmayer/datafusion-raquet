use std::sync::Arc;

use datafusion::catalog::TableProvider;

use datafusion::error::{DataFusionError, Result};
use datafusion_sql::ResolvedTableReference;

use datafusion::execution::SessionState;

pub async fn resolve_table_provider(
    state: &SessionState,
    table_ref: &ResolvedTableReference,
) -> datafusion::common::Result<Option<Arc<dyn TableProvider>>> {
    let table = table_ref.table.as_ref();
    let schema = state.schema_for_ref(table_ref.clone())?;
    let table_provider = schema.table(&table).await?;
    Ok(table_provider)
}
