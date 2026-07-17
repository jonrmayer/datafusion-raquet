// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use std::fmt::Debug;
use std::sync::Arc;
use std::any::Any;

use async_trait::async_trait;
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::catalog::{Session, TableProviderFactory};
use datafusion::common::DFSchema;
use datafusion::datasource::listing::ListingTableUrl;
use datafusion::datasource::{TableProvider, TableType};
use datafusion::error::{DataFusionError, Result as DfResult};
use datafusion::logical_expr::utils::conjunction;
use datafusion::logical_expr::{CreateExternalTable, Expr, TableProviderFilterPushDown};
use datafusion::physical_plan::ExecutionPlan;
use object_store::ObjectMeta;

use super::config::RaquetTableConfig;
use crate::tables::config::RaquetTableUrl;
use datafusion::datasource::physical_plan::FileScanConfigBuilder;
use datafusion::execution::object_store::ObjectStoreUrl;
use datafusion_datasource_parquet::source::ParquetSource;

use datafusion::datasource::memory::DataSourceExec;
use datafusion_datasource::PartitionedFile;

/// The table provider for raquet stores.
pub struct RaquetTable {
    table_config: RaquetTableConfig,
}

impl Debug for RaquetTable {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl RaquetTable {
    pub fn new(table_config: RaquetTableConfig) -> Self {
        Self { table_config }
    }

    pub fn table_config(&self) -> RaquetTableConfig {
        self.table_config.clone()
    }

    pub async fn from_path(path: String) -> Self {
        let table_url = ListingTableUrl::parse(path).unwrap();
        let raquet_url = RaquetTableUrl::RaquetStore(table_url);
        let schema = raquet_url.infer_schema().await.unwrap();
        let table_config = RaquetTableConfig::new(raquet_url, schema);
        Self { table_config }
    }
}

impl RaquetTable {
    pub async fn get_partitioned_file(&self) -> PartitionedFile {
        let (_store, object_meta) = self
            .table_config()
            .get_table_url()
            .get_store_location()
            .await
            .unwrap();
        PartitionedFile::new_from_meta(object_meta)
    }

    pub async fn get_object_meta(&self) -> ObjectMeta {
        let (_store, object_meta) = self
            .table_config()
            .get_table_url()
            .get_store_location()
            .await
            .unwrap();
        object_meta
    }

    pub async fn get_object_store_url(&self) -> ObjectStoreUrl {
        let object_url = self
            .table_config()
            .get_table_url()
            .get_object_store_url()
            .await
            .unwrap();
        object_url
    }
}

#[async_trait]
impl TableProvider for RaquetTable {
        fn as_any(&self) -> &dyn Any {
        self
    }
    fn schema(&self) -> SchemaRef {
        self.table_config.get_schema_ref()
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    // there's no projected columns or partitions with the zarr data,
    // so really all we have are arrays that are present in all the data
    // chunks. there's not much to check here, we do use the filter
    // pushdown to avoid reading entire chunk, so pretty much all the
    // available arrays can be used as Inexact filters.
    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> datafusion::error::Result<Vec<TableProviderFilterPushDown>> {
        Ok(vec![TableProviderFilterPushDown::Inexact; filters.len()])
    }

    async fn scan(
        &self,
        state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> datafusion::error::Result<Arc<dyn ExecutionPlan>> {
        let df_schema = DFSchema::try_from(self.schema())?;
        let predicate = conjunction(filters.to_vec());
        let predicate = predicate
            .map(|predicate| state.create_physical_expr(predicate, &df_schema))
            .transpose()?
            // if there are no filters, use a literal true to have a predicate
            // that always evaluates to true we can pass to the index
            .unwrap_or_else(|| datafusion::physical_expr::expressions::lit(true));

        let object_store_url = self.get_object_store_url().await;
        let object_meta = self.get_object_meta().await;

        let source = Arc::new(ParquetSource::new(self.schema()).with_predicate(predicate));

        let scan_config_builder = FileScanConfigBuilder::new(object_store_url, source)
            .with_file(PartitionedFile::new_from_meta(object_meta.clone()))
            .with_limit(limit)
            .with_projection(projection.cloned());
        Ok(DataSourceExec::from_data_source(
            scan_config_builder.build(),
        ))
    }
}

/// The factory for the zarr table.
#[derive(Debug)]
pub struct RaquetTableFactory {}

#[async_trait]
impl TableProviderFactory for RaquetTableFactory {
    async fn create(
        &self,
        _state: &dyn Session,
        cmd: &CreateExternalTable,
    ) -> DfResult<Arc<dyn TableProvider>> {
        let table_url = match cmd.file_type.as_str() {
            "RAQUET_STORE" => RaquetTableUrl::RaquetStore(ListingTableUrl::parse(&cmd.location)?),
            // #[cfg(feature = "icechunk")]
            // "ICECHUNK_REPO" => ZarrTableUrl::IcechunkRepo(ListingTableUrl::parse(&cmd.location)?),
            _ => {
                return Err(DataFusionError::Execution(format!(
                    "Unsupported file type {}",
                    cmd.file_type
                )));
            }
        };

        let inferred_schema = table_url.infer_schema().await?;
        // if cmd.schema.fields().is_empty() {
        let schema = inferred_schema;
        // };
        // } else {
        //     let provided_schema: Schema = cmd.schema.as_ref().into();
        //     for field in provided_schema.fields() {
        //         let target_type = inferred_schema.field_with_name(field.name())?.data_type();
        //         if field.data_type() != target_type {
        //             return Err(DataFusionError::Execution(format!(
        //                 "Requested column {}'s type does not match data from store",
        //                 field.name()
        //             )));
        //         }
        //     }

        //     Arc::new(provided_schema)
        // };

        let raquet_config = RaquetTableConfig::new(table_url, schema);
        let table_provider = RaquetTable::new(raquet_config);
        Ok(Arc::new(table_provider))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_https() {
        let path_or_url =
            "https://storage.googleapis.com/raquet_demo_data/spain_solar_ghi.parquet".to_string();
        let rt = RaquetTable::from_path(path_or_url).await;

        println!("{:?}", rt.table_config());
    }
    #[tokio::test]
    async fn test_local() {
        let path_or_url = "file:///home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet".to_string();
        let rt = RaquetTable::from_path(path_or_url).await;

        println!("{:?}", rt.table_config());
    }
}
