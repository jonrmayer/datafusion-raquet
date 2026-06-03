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

use std::path::PathBuf;
use std::sync::Arc;

use arrow_schema::{Fields, Schema, SchemaRef};
use datafusion::datasource::listing::ListingTableUrl;
use datafusion::error::{DataFusionError, Result as DfResult};
use object_store::path::Path;

use object_store::local::LocalFileSystem;
use object_store::{ObjectMeta, ObjectStore, ObjectStoreExt};

use crate::RaquetMetadataReader;

/// A raquet table configuration.
#[derive(Clone, Debug)]
pub struct RaquetTableConfig {
    schema_ref: SchemaRef,
    table_url: RaquetTableUrl,
    projection: Option<Vec<usize>>,
}

impl RaquetTableConfig {
    pub(crate) fn new(table_url: RaquetTableUrl, schema_ref: SchemaRef) -> Self {
        Self {
            schema_ref,
            table_url,
            projection: None,
        }
    }

    pub(crate) fn with_projection(mut self, projection: Vec<usize>) -> Self {
        self.projection = Some(projection);
        self
    }

    pub(crate) fn get_projection(&self) -> Option<Vec<usize>> {
        self.projection.clone()
    }

    pub(crate) fn get_table_url(&self) -> RaquetTableUrl {
        self.table_url.clone()
    }

    pub(crate) fn get_schema_ref(&self) -> SchemaRef {
        self.schema_ref.clone()
    }

    pub(crate) fn get_projected_schema_ref(&self) -> SchemaRef {
        if let Some(projection) = &self.projection {
            let projected_fields: Fields = projection
                .iter()
                .map(|&i| self.schema_ref.field(i).clone())
                .collect();
            Arc::new(Schema::new(projected_fields))
        } else {
            self.schema_ref.clone()
        }
    }
}

/// We can create a table based on a directory with a supported zarr
/// file/folder structure, or from an icechunk repo.
#[derive(Clone, Debug)]
pub(crate) enum RaquetTableUrl {
    RaquetStore(ListingTableUrl),
}

impl RaquetTableUrl {
    pub(crate) async fn get_store_location(&self) -> DfResult<(Arc<dyn ObjectStore>, ObjectMeta)> {
        match self {
            Self::RaquetStore(table_url) => match table_url.scheme() {
                "file" => {
                    let path = PathBuf::from("/".to_owned() + table_url.prefix().as_ref());
                    let storage_container = LocalFileSystem::new();
                    let location = Path::from_filesystem_path(path).unwrap();
                    let object_store: Arc<dyn ObjectStore> = Arc::new(storage_container);
                    let meta = object_store.head(&location).await.unwrap();
                    Ok((object_store, meta))
                }
                // "s3" => {
                //     let store = AmazonS3Builder::from_env()
                //         .with_url(table_url.get_url().as_str())
                //         .build()?;
                //     let store = AsyncObjectStore::new(store);
                //     Ok((Arc::new(store), Some(table_url.prefix().to_string())))
                // }
                _ => Err(DataFusionError::Execution(format!(
                    "Unsupported table url scheme {} for raquet store",
                    table_url.scheme()
                ))),
            },
        }
    }

    pub(crate) async fn infer_schema(&self) -> DfResult<SchemaRef> {
        let (store, location) = self.get_store_location().await?;
        let rdr = RaquetMetadataReader::new(location, store);
        let schema = rdr.get_raquet_schema().await;
        Ok(schema)
    }
}

// #[cfg(test)]
// mod zarr_config_tests {
//     use super::*;
//     #[cfg(feature = "icechunk")]
//     use crate::test_utils::get_local_icechunk_repo;
//     use crate::test_utils::get_local_zarr_store;

//     #[tokio::test]
//     async fn schema_inference_tests() {
//         // local zarr directory.
//         let (wrapper, schema) = get_local_zarr_store(true, 0.0, "data_for_config_dir").await;
//         let path = wrapper.get_store_path();

//         let table_url = ListingTableUrl::parse(path).unwrap();
//         let zarr_table_url = ZarrTableUrl::ZarrStore(table_url);
//         let inferred_schema = zarr_table_url.infer_schema().await.unwrap();
//         assert_eq!(inferred_schema, schema);

//         // local icechunk repo.
//         #[cfg(feature = "icechunk")]
//         {
//             let (wrapper, schema) =
//                 get_local_icechunk_repo(true, 0.0, "data_for_config_repo").await;
//             let path = wrapper.get_store_path();

//             let table_url = ListingTableUrl::parse(path).unwrap();
//             let zarr_table_url = ZarrTableUrl::IcechunkRepo(table_url);
//             let inferred_schema = zarr_table_url.infer_schema().await.unwrap();
//             assert_eq!(inferred_schema, schema);
//         }
//     }
// }
