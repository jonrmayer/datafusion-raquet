use std::collections::HashMap;
use std::sync::Arc;

use arrow_schema::SchemaRef;
use async_trait::async_trait;
use datafusion::catalog::Session;
use datafusion::common::{GetExt, Statistics};
use datafusion::config::{ConfigField, ConfigFileType, TableParquetOptions};
use datafusion::datasource::file_format::FileFormat;
use datafusion::datasource::file_format::file_compression_type::FileCompressionType;
use datafusion::datasource::physical_plan::{FileScanConfig, FileSinkConfig, FileSource};

use datafusion::error::Result;
use datafusion::physical_expr::LexRequirement;
use datafusion::physical_plan::ExecutionPlan;
use datafusion_datasource::TableSchema;
use datafusion_datasource::file_format::FileFormatFactory;
use datafusion_datasource::file_scan_config::FileScanConfigBuilder;
use datafusion_datasource_parquet::ParquetFormat;
use datafusion_datasource_parquet::source::ParquetSource;

use object_store::{ObjectMeta, ObjectStore};

use crate::RaquetMetadataReader;
use crate::RaquetSource;

#[derive(Default, Debug)]
pub struct RaquetFormatFactory {
    /// inner options for parquet
    pub options: Option<TableParquetOptions>,
}

impl RaquetFormatFactory {
    /// Creates an instance of [RaquetFormatFactory]
    pub fn new() -> Self {
        Self { options: None }
    }

    /// Creates an instance of [GeoParquetFormatFactory] with customized default options
    pub fn new_with_options(options: TableParquetOptions) -> Self {
        Self {
            options: Some(options),
        }
    }
}

impl FileFormatFactory for RaquetFormatFactory {
    fn create(
        &self,
        state: &dyn Session,
        format_options: &HashMap<String, String>,
    ) -> Result<Arc<dyn FileFormat>> {
        let parquet_options = match &self.options {
            None => {
                let mut table_options = state.default_table_options();
                table_options.set_config_format(ConfigFileType::PARQUET);
                table_options.alter_with_string_hash_map(format_options)?;
                table_options.parquet
            }
            Some(parquet_options) => {
                let mut parquet_options = parquet_options.clone();
                for (k, v) in format_options {
                    parquet_options.set(k, v)?;
                }
                parquet_options
            }
        };

        let parquet_format = ParquetFormat::default().with_options(parquet_options);
        Ok(Arc::new(RaquetFormat::new(parquet_format)))
    }

    fn default(&self) -> Arc<dyn FileFormat> {
        Arc::new(RaquetFormat::default())
    }

   
}

impl GetExt for RaquetFormatFactory {
    fn get_ext(&self) -> String {
        "parquet".to_string()
    }
}

/// GeoParquet `FileFormat` implementation
#[derive(Debug, Default)]
pub struct RaquetFormat {
    inner: ParquetFormat,
    parse_to_native: bool,
}

impl RaquetFormat {
    /// Creates a new instance of `RaquetFormat`
    pub fn new(format: ParquetFormat) -> Self {
        Self {
            inner: format.with_skip_metadata(false),
            parse_to_native: false,
        }
    }
}

impl RaquetFormat {
    async fn load_schema(
        &self,
        state: &dyn Session,
        store: &Arc<dyn ObjectStore>,
        objects: &[ObjectMeta],
    ) -> Result<SchemaRef> {
        let file_metadata_cache = state.runtime_env().cache_manager.get_file_metadata_cache();
        // file_metadata_cache.

        for e in file_metadata_cache.list_entries() {
            // e.1.extra.
            println!("{:?}", e.0)
        }

        let rdr = RaquetMetadataReader::new(objects[0].clone(), store.clone());
        let schema = rdr.get_raquet_schema().await;

        Ok(schema)
    }
}

#[async_trait]
impl FileFormat for RaquetFormat {
   
    fn get_ext(&self) -> String {
        self.inner.get_ext()
    }

    fn get_ext_with_compression(
        &self,
        file_compression_type: &FileCompressionType,
    ) -> Result<String> {
        self.inner.get_ext_with_compression(file_compression_type)
    }

    fn compression_type(&self) -> Option<FileCompressionType> {
        self.inner.compression_type()
    }

    async fn infer_schema(
        &self,
        state: &dyn Session,
        store: &Arc<dyn ObjectStore>,
        objects: &[ObjectMeta],
    ) -> Result<SchemaRef> {
        // let schema = self.inner.infer_schema(state, store, objects).await?;
        let schema = self.load_schema(state, store, objects).await?;

        Ok(schema)
    }

    async fn infer_stats(
        &self,
        _state: &dyn Session,
        store: &Arc<dyn ObjectStore>,
        table_schema: SchemaRef,
        object: &ObjectMeta,
    ) -> Result<Statistics> {
        self.inner
            .infer_stats(_state, store, table_schema, object)
            .await
    }

    async fn create_physical_plan(
        &self,
        _state: &dyn Session,
        conf: FileScanConfig,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let source = conf.file_source().clone();
        let geoparquet_source = source.downcast_ref::<RaquetSource>().unwrap();
        let parquet_source = &geoparquet_source.inner;

        let file_scan_config_builder =
            FileScanConfigBuilder::from(conf).with_source(Arc::new(parquet_source.clone()));
        let new_conf = file_scan_config_builder.build();

        self.inner.create_physical_plan(_state, new_conf).await
    }

    async fn create_writer_physical_plan(
        &self,
        _input: Arc<dyn ExecutionPlan>,
        _state: &dyn Session,
        _conf: FileSinkConfig,
        _order_requirements: Option<LexRequirement>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        todo!("writing not implemented for GeoParquet yet")
    }

    fn file_source(&self, table_schema: TableSchema) -> Arc<dyn FileSource> {
        let parquet_source = self.inner.file_source(table_schema);
        // safe to do unwrap here because the inner type is ParquetSource for sure
        let inner = parquet_source
           
            .downcast_ref::<ParquetSource>()
            .unwrap();
        Arc::new(RaquetSource {
            inner: inner.clone(),
        })
    }
}
