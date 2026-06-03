use std::sync::Arc;



use datafusion::datasource::TableProvider;

use datafusion::logical_expr::Expr;


use datafusion::catalog::TableFunctionImpl;

use datafusion::error::Result;


use datafusion::common::{ScalarValue, plan_err};



use datafusion_python_util::get_global_ctx;



pub fn read_raquet(// catalog_list: Arc<dyn CatalogProviderList>,
    // options: &ConfigOptions,
) -> Arc<dyn TableFunctionImpl + 'static> {
    Arc::new(ReadRaquet::new())
}

#[derive(Debug, Clone)]
pub struct ReadRaquet {
    // catalog_list: Option<Arc<dyn CatalogProviderList>>,
    // config_options: Option<ConfigOptions>,
}
impl ReadRaquet {
    pub fn new() -> Self {
        Self {
            // catalog_list: None,
            // config_options: None,
        }
    }

    // pub fn catalog_list(&self) -> Arc<dyn CatalogProviderList> {
    //     self.catalog_list.clone().unwrap()
    // }

    // pub fn config_options(&self) -> ConfigOptions {
    //     self.config_options.clone().unwrap()
    // }
}

impl TableFunctionImpl for ReadRaquet {
    fn call(&self, args: &[Expr]) -> Result<Arc<dyn TableProvider>> {
        let ctx = get_global_ctx().clone();

        // let state = ctx.state();
        // let options = state.config_options();
        // // state.catalog_list().
        // // ctx.state().catalog_list()
        let Some(Expr::Literal(ScalarValue::Utf8(Some(table_name)), _)) = args.first() else {
            return plan_err!("read_raquet requires at least one string argument");
        };
        let table_provider = futures::executor::block_on(ctx.table_provider(table_name)).unwrap();
        // .map_err(|e| e.context(format!("couldn't get table '{}'", table_ref.table)))?
        // .ok_or_else(|| DataFusionError::Plan(format!("no such table {}", table_ref.schema)));
        // let table_provider = ctx.table_provider(table_name).await.unwrap();
        Ok(table_provider)
    }

    // let table_ref = TableReference::from(table_name).resolve(
    //     &self.config_options().catalog.default_catalog,
    //     &self.config_options().catalog.default_schema,
    // );

    // let table_provider = util::get_table(self.catalog_list().as_ref(), &table_ref)
    //     .map_err(|e| DataFusionError::Plan(e.to_string()))?;

    // let table_schema = table_provider.schema();
    // let filtered_columns = table_schema
    //     .fields()
    //     .iter()
    //     .filter(|&x| *x.name() != "metadata".to_string())
    //     .map(|f| col(f.name()))
    //     .collect::<Vec<Expr>>();

    // let table_source = provider_as_source(table_provider);
    // let builder = LogicalPlanBuilder::scan(table_ref, table_source, None)?;

    // let logical_plan = builder
    //     .filter(col("block").not_eq(lit(0)))?
    //     .project(filtered_columns)?
    //     .build()?;
    // let vt = ViewTable::new(logical_plan, None);

    // Ok(Arc::new(vt))
    // }
    // }
}

// #[async_trait]
// impl TableProvider for ReadRaquetTableProvider {
//     fn as_any(&self) -> &dyn std::any::Any {
//         self
//     }

//     fn schema(&self) -> SchemaRef {
//         self.schema.clone()
//     }

//     fn table_type(&self) -> TableType {
//         TableType::Base
//     }

//     async fn scan(
//         &self,
//         _state: &dyn Session,
//         projection: Option<&Vec<usize>>,
//         _filters: &[Expr],
//         _limit: Option<usize>,
//     ) -> Result<Arc<dyn ExecutionPlan>> {
//         let batches = self.batches.clone();
//         Ok(MemorySourceConfig::try_new_exec(
//             &[batches],
//             TableProvider::schema(self),
//             projection.cloned(),
//         )?)
//     }
// }

// pub fn read_raquet_batches(raquet_path: impl AsRef<Path>) -> Result<(SchemaRef, Vec<RecordBatch>)> {
//     let mut file = File::open(raquet_path)?;

//     let parquet_reader = ParquetRecordBatchReaderBuilder::try_new(file)
//         .unwrap()
//         .with_batch_size(8192)
//         .build()
//         .unwrap();

//     let mut batches = Vec::new();

//     for batch in parquet_reader {
//         let b = batch.unwrap();
//         if b.num_rows() > 1 {
//             batches.push(b);
//         }
//     }
//     let schema = batches[0].schema();
//     // let schema = Arc::new(schema);
//     Ok((schema, batches))
// }
