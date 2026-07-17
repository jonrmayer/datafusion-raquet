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

use object_store::http::HttpBuilder;
use url::Url;

use arrow_schema::{Fields, Schema, SchemaRef};
use datafusion::datasource::listing::ListingTableUrl;
use datafusion::error::{DataFusionError, Result as DfResult};
use object_store::path::Path;

use datafusion::execution::object_store::ObjectStoreUrl;
use object_store::local::LocalFileSystem;
use object_store::{ClientOptions, ObjectMeta, ObjectStore, ObjectStoreExt, parse_url_opts};

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
    pub(crate) async fn get_object_store_url(&self) -> DfResult<ObjectStoreUrl> {
        match self {
            Self::RaquetStore(table_url) => match table_url.scheme() {
                "file" => {
                    let object_store_url = ObjectStoreUrl::parse("file://")?;
                    Ok(object_store_url)
                }
                "http" | "https" => {
                    let object_store_url = ObjectStoreUrl::parse("https://storage.googleapis.com")?;
                    Ok(object_store_url)
                }
                _ => Err(DataFusionError::Execution(format!(
                    "Unsupported table url scheme {} for raquet store",
                    table_url.scheme()
                ))),
                // let url = Url::parse(table_url.get_url().as_str()).unwrap();
                // let object_store_url =
                //     ObjectStoreUrl::parse(url.origin().ascii_serialization()).unwrap();

                // Ok(object_store_url)
            },
        }
    }
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
                "http" | "https" => {
                    let url = Url::parse(table_url.get_url().as_str()).unwrap();
                    let base_url = format!("{}/{}", url.scheme(), url.domain().unwrap());
                    let location = Path::from("/".to_owned() + table_url.prefix().as_ref());
                    let options = ClientOptions::new().with_allow_http(true);
                    let object_store_url =
                        ObjectStoreUrl::parse(url.origin().ascii_serialization()).unwrap();

                    let storage_container = HttpBuilder::new()
                        .with_url(object_store_url.as_str())
                        .with_client_options(options)
                        .build()
                        .unwrap();
                    let object_store: Arc<dyn ObjectStore> = Arc::new(storage_container);
                    // let location = Path::from("path/to/blob.parquet");

                    // let options = [("allow_http", "true")];
                    // let (box_store, location) = parse_url_opts(&url, options)?;

                    // let object_store: Arc<dyn ObjectStore> = Arc::from(box_store);
                    let meta = object_store.head(&location).await?;

                    Ok((object_store, meta))
                }

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

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test() {
        let path_or_url = "https://storage.googleapis.com/raquet_demo_data/spain_solar_ghi.parquet";
        let table_url = ListingTableUrl::parse(path_or_url).unwrap();
        let raquet_url = RaquetTableUrl::RaquetStore(table_url);
        let lt = raquet_url.get_object_store_url().await.unwrap();
         let schema = raquet_url.infer_schema().await.unwrap();
        let table_config = RaquetTableConfig::new(raquet_url, schema);

        // let url = Url::parse(lt.get_url().as_str()).unwrap();
        // let base_url = format!("{}/{}", url.scheme(), url.domain().unwrap());

       println!("{:?} {:?}", table_config, lt);
    }

    #[tokio::test]
    async fn test_https() {
        let path_or_url = "https://storage.googleapis.com/raquet_demo_data/spain_solar_ghi.parquet";
        let table_url = ListingTableUrl::parse(path_or_url).unwrap();
        let raquet_url = RaquetTableUrl::RaquetStore(table_url);
        let schema = raquet_url.infer_schema().await.unwrap();
        let table_config = RaquetTableConfig::new(raquet_url, schema);

        println!("{:?}", table_config);
    }
    #[tokio::test]
    async fn test_local() {
        let path_or_url = "file:///home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet";
        let table_url = ListingTableUrl::parse(path_or_url).unwrap();
        let raquet_url = RaquetTableUrl::RaquetStore(table_url);
        let lt = raquet_url.get_object_store_url().await.unwrap();
        let schema = raquet_url.infer_schema().await.unwrap();

        let table_config = RaquetTableConfig::new(raquet_url, schema);

        println!("{:?} {:?}", table_config, lt);
    }
}
