use std::ops::Deref;
use std::sync::Arc;

use async_trait::async_trait;

use datafusion::arrow::datatypes::{Schema, SchemaRef};
use datafusion::arrow::record_batch::RecordBatch;

use datafusion::datasource::{TableProvider, TableType};

use datafusion::logical_expr::Expr;

use datafusion::config::ConfigOptions;

use datafusion::catalog::view::ViewTable;
use datafusion::catalog::{CatalogProviderList, Session, TableFunctionImpl};
use datafusion::common::DataFusionError;
use datafusion::physical_plan::ExecutionPlan;

use datafusion::datasource::memory::MemorySourceConfig;
use datafusion::error::Result;

use datafusion_sql::TableReference;

use datafusion::common::{ScalarValue, plan_err};

use datafusion::datasource::provider_as_source;

use datafusion::logical_expr::{LogicalPlanBuilder, LogicalTableSource, col, lit, table_scan};

use crate::views::util;

use crate::RaquetTable;

pub fn read_raquet_metadata(
    catalog_list: Arc<dyn CatalogProviderList>,  
    options: &ConfigOptions,
) -> Arc<dyn TableFunctionImpl + 'static> {
    Arc::new(ReadRaquetMetadata::new(
        catalog_list,       
        options,
    ))
}

#[derive(Debug)]
pub struct ReadRaquetMetadata {
    catalog_list: Arc<dyn CatalogProviderList>,
    config_options: ConfigOptions,
}
impl ReadRaquetMetadata {
    fn new(catalog_list: Arc<dyn CatalogProviderList>, config_options: &ConfigOptions) -> Self {
        Self {
            catalog_list,
            config_options: config_options.clone(),
        }
    }
}

impl TableFunctionImpl for ReadRaquetMetadata {
    fn call(&self, args: &[Expr]) -> Result<Arc<dyn TableProvider>> {
        let Some(Expr::Literal(ScalarValue::Utf8(Some(table_name)), _)) = args.first() else {
            return plan_err!("read_raquet requires at least one string argument");
        };

        let table_ref = TableReference::from(table_name).resolve(
            &self.config_options.catalog.default_catalog,
            &self.config_options.catalog.default_schema,
        );

        let table_provider = util::get_table(self.catalog_list.as_ref(), &table_ref)
            .map_err(|e| DataFusionError::Plan(e.to_string()))?;

        let table_schema = table_provider.schema();
        let filtered_columns = table_schema
            .fields()
            .iter()
            .filter(|&x| *x.name() == "metadata".to_string())
            .map(|f| col(f.name()))
            .collect::<Vec<Expr>>();

        let table_source = provider_as_source(table_provider);
       

        let builder = LogicalPlanBuilder::scan(table_ref, table_source, None)?;
     
        let logical_plan = builder
            .filter(col("block").eq(lit(0)))?
            .project(filtered_columns)?
            .build()?;
        let vt = ViewTable::new(logical_plan, None);      
        Ok(Arc::new(vt))

    }
   
}
