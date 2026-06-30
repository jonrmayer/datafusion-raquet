use std::sync::Arc;

use datafusion::datasource::TableProvider;
use datafusion::execution::SessionState;

use datafusion::logical_expr::Expr;

use datafusion::catalog::view::ViewTable;
use datafusion::catalog::{TableFunctionArgs, TableFunctionImpl};
use datafusion::common::DataFusionError;

use datafusion::error::Result;

use datafusion::optimizer::analyzer::resolve_grouping_function;
use datafusion::physical_plan::collect;
use datafusion_sql::TableReference;

use datafusion::common::{ScalarValue, plan_err};

use datafusion::datasource::provider_as_source;

use datafusion::logical_expr::{LogicalPlanBuilder, col, lit};
use tokio::{runtime::Handle, task::block_in_place};

use crate::views::util::resolve_table_provider;

use crate::error::RaquetDataFusionResult;
use quadbin_geo_rs::GeoCells;
use quadbin_schema::Metadata;

fn geocells(metadata: Metadata, wkt: &String) -> RaquetDataFusionResult<Expr> {
    let resolution = metadata.max_zoom().clone();
    let geolist = GeoCells::new(wkt.clone(), resolution as i8)
        .intersecting_cells()?
        .iter()
        .map(|x| lit(*x))
        .collect();
    let expr = col("block").in_list(geolist, false);

    Ok(expr)
}

#[derive(Debug, Clone)]
pub struct ReadRaquetAt {}

impl TableFunctionImpl for ReadRaquetAt {
    fn call_with_args(&self, args: TableFunctionArgs) -> Result<Arc<dyn TableProvider>> {
        let exprs = args.exprs();

        let Some(Expr::Literal(ScalarValue::Utf8(Some(table_name)), _)) = exprs.get(0) else {
            return plan_err!("read_raquet_at requires a table_name string argument");
        };
        let Some(Expr::Literal(ScalarValue::Utf8(Some(wkt)), _)) = exprs.get(1) else {
            return plan_err!("read_raquet_at requires a wkt string argument");
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

        let table_provider = resolve_table_provider(state, &table_ref)?;

        
        // let table_provider = Handle::current().block_on(async {
        //     let table = table_ref.table.as_ref();
        //     let schema = state.schema_for_ref(table_ref.clone()).unwrap();
        //     let table_provider = schema.table(&table).await.unwrap().unwrap();
        //     table_provider
        // });
        //  let Some(table_provider) = block_in_place(|| {
        //     Handle::current().block_on(resolve_table_provider(state, &table_ref))
        // })?
        // else {
        //     todo!()
        // };
        // let Some(table_provider) = spawn_blocking(move || {
        //     Handle::current().block_on(resolve_table_provider(state, &table_ref))
        // })?
        // else {
        //     todo!()
        // };
        let table_schema = table_provider.schema();
        let block_metadata =
            Metadata::try_from(table_schema.field_with_name("block")?.as_ref()).unwrap_or_default();

        let filtered_columns = table_schema
            .fields()
            .iter()
            .filter(|&x| *x.name() != "metadata".to_string())
            .map(|f| col(f.name()))
            .collect::<Vec<Expr>>();

        let table_source = provider_as_source(table_provider);

        let builder = LogicalPlanBuilder::scan(table_ref, table_source, None)?;

        let logical_plan = builder
            .filter(geocells(block_metadata, wkt)?)?
            .project(filtered_columns)?
            .build()?;
        let vt = ViewTable::new(logical_plan, None);
        Ok(Arc::new(vt))
    }
}
