use std::sync::Arc;

use datafusion::catalog::TableProvider;

use datafusion_sql::ResolvedTableReference;

use datafusion::execution::SessionState;

use datafusion::error::DataFusionError;

pub  fn resolve_table_provider(
    state: &SessionState,
    table_ref: &ResolvedTableReference,
) -> datafusion::common::Result<Arc<dyn TableProvider>> {
    let table = table_ref.table.as_ref();
    let schema = state.schema_for_ref(table_ref.clone())?;
    // let table_provider = schema.table(&table).await?;
    // Ok(table_provider)
     futures::executor::block_on(schema.table(table))
        .map_err(|e| e.context(format!("couldn't get table '{}'", table_ref.table)))?
        .ok_or_else(|| DataFusionError::Plan(format!("no such table {}", table_ref.schema)))
}
