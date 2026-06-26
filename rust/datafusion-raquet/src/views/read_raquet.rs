use std::sync::Arc;

use datafusion::datasource::TableProvider;
use datafusion::execution::SessionState;

use datafusion::logical_expr::Expr;

use datafusion::catalog::view::ViewTable;
use datafusion::catalog::{TableFunctionArgs, TableFunctionImpl};
use datafusion::common::DataFusionError;

use datafusion::error::Result;

use datafusion_sql::TableReference;

use datafusion::common::{ScalarValue, plan_err};

use datafusion::datasource::provider_as_source;

use datafusion::logical_expr::{LogicalPlanBuilder, col, lit};
use tokio::{runtime::Handle, task::block_in_place};

use crate::views::util::resolve_table_provider;

#[derive(Debug, Clone)]
pub struct ReadRaquet {}

impl TableFunctionImpl for ReadRaquet {
    fn call_with_args(&self, args: TableFunctionArgs) -> Result<Arc<dyn TableProvider>> {
        let exprs = args.exprs();
        let Some(Expr::Literal(ScalarValue::Utf8(Some(table_name)), _)) = exprs.first() else {
            return plan_err!("read_raquet requires a table_name string argument");
        };
        let state = args
            .session()
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| DataFusionError::Internal("failed to downcast state".into()))?;
        let config_options = state.config_options();
        let table_ref = TableReference::from(table_name).resolve(
            &config_options.catalog.default_catalog,
            &config_options.catalog.default_schema,
        );
        let Some(table_provider) = block_in_place(|| {
            Handle::current().block_on(resolve_table_provider(state, &table_ref))
        })?
        else {
            todo!()
        };
        let table_schema = table_provider.schema();
        let filtered_columns = table_schema
            .fields()
            .iter()
            .filter(|&x| *x.name() != "metadata".to_string())
            .map(|f| col(f.name()))
            .collect::<Vec<Expr>>();

        let table_source = provider_as_source(table_provider);

        let builder = LogicalPlanBuilder::scan(table_ref, table_source, None)?;

        let logical_plan = builder
            .filter(col("block").not_eq(lit(0)))?
            .project(filtered_columns)?
            .build()?;
        let vt = ViewTable::new(logical_plan, None);
        Ok(Arc::new(vt))
    }
}
