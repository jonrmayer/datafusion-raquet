mod format;

mod location;
mod metadata_filter;

use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

pub use format::RaquetFormat;
pub use location::MetaDataLocation;
pub use metadata_filter::metadata_arrow_predicate;

use object_store::{ObjectMeta, ObjectStore};

use parquet::arrow::arrow_reader::{ArrowReaderMetadata, ArrowReaderOptions};
use parquet::arrow::async_reader::ParquetObjectReader;
use parquet::arrow::async_reader::ParquetRecordBatchStreamBuilder;

use rastertile_rs::{BinaryType, CompressionFormat, RasterDataType};

use parquet::errors::Result;

use arrow_array::cast::as_string_array;
use futures::TryStreamExt;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use arrow_schema::{Field, FieldRef, Schema, SchemaRef};

use rasterarrow_schema::error::RasterArrowResult;
use rasterarrow_schema::{Metadata, RasterArrowType, RasterType};


#[derive(Debug, Clone)]
pub struct RaquetMetadataReader {
    pub meta: ObjectMeta,
    pub store: Arc<dyn ObjectStore>,
}

impl RaquetMetadataReader {
    pub fn new(meta: ObjectMeta, store: Arc<dyn ObjectStore>) -> Self {
        Self { meta, store }
    }
    pub fn store(&self) -> Arc<dyn ObjectStore> {
        self.store.clone()
    }

    pub fn meta(&self) -> ObjectMeta {
        self.meta.clone()
    }

    pub async fn get_raquet_schema(&self) -> SchemaRef {
        let (raquet_format, existing_schema) = self.get_format_and_schema().await.unwrap();
        let raquet_metadata = self.get_raquet_metadata(raquet_format).unwrap();

        let new_schema = infer_rasterarrow_schema(&existing_schema, &raquet_metadata).unwrap();
        new_schema
    }

    fn get_raquet_metadata(&self, raquet_format: RaquetFormat) -> Result<RaquetMetadata> {
        let mut columns: HashMap<String, RaquetColumnMetadata> = HashMap::new();
        let version = raquet_format.version().unwrap();
        let tile_size = raquet_format.tiling().unwrap().block_height.unwrap() as usize;
        // println!("tile_size{:?}", tile_size);
        let compression_str = raquet_format.compression().unwrap();
        let compression = CompressionFormat::from_str(&compression_str).unwrap();
        let bands = raquet_format.bands().unwrap();

        for (_, band) in bands.iter().enumerate() {
            let name = band.name.clone().unwrap();
            let dtype = band.r#type.clone().unwrap();
            let rdt = RasterDataType::from_str(&dtype).unwrap();
            let rcm = RaquetColumnMetadata::new(tile_size, rdt, compression);
            columns.insert(name, rcm);
        }
        let rm = RaquetMetadata { version, columns };

        return Ok(rm);
    }

    async fn get_format_and_schema(&self) -> Result<(RaquetFormat, Schema)> {
        let object_reader = ParquetObjectReader::new(self.store(), self.meta().location)
            .with_file_size(self.meta().size);
        let builder = ParquetRecordBatchStreamBuilder::new(object_reader.clone())
            .await
            .unwrap();

        // ParquetRecordBatchStreamBuilder::new_with_metadata(input, metadata)
        let metadata = builder.metadata().clone();
        let reader_metadata =
            ArrowReaderMetadata::try_new(metadata, ArrowReaderOptions::default()).unwrap();
        let existing_schema = reader_metadata.schema().clone();
        let parquet_meta = reader_metadata.metadata().clone();
        let metadata_location = MetaDataLocation::new(parquet_meta);
        let format_stream = builder
            .with_row_groups(metadata_location.row_group_indexes())
            .with_row_filter(metadata_location.row_filter())
            .with_projection(metadata_location.projection())
            .build()
            .unwrap();
        let batches = format_stream.try_collect::<Vec<_>>().await.unwrap();

        let raquet_str = batches
            .first()
            .map(|rb| as_string_array(rb.column(0)).value(0))
            .unwrap();

        let raquet_format: RaquetFormat = serde_json::from_str(raquet_str).unwrap();
        Ok((raquet_format, existing_schema.deref().clone()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RaquetMetadata {
    /// The version identifier for the GeoParquet specification.
    pub version: String,

    /// Metadata about geometry columns. Each key is the name of a geometry column in the table.
    pub columns: HashMap<String, RaquetColumnMetadata>,
}

/// GeoParquet column metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RaquetColumnMetadata {
    pub tile_size: usize,
    pub binary_type: BinaryType,
    pub data_type: RasterDataType,
    pub compression: CompressionFormat,
}

impl RaquetColumnMetadata {
    pub fn new(
        tile_size: usize,
        data_type: RasterDataType,
        compression: CompressionFormat,
    ) -> Self {
        RaquetColumnMetadata {
            tile_size: tile_size,
            binary_type: BinaryType::Separated,
            data_type: data_type,
            compression: compression,
        }
    }
}

impl From<RaquetColumnMetadata> for Metadata {
    fn from(value: RaquetColumnMetadata) -> Self {
        Metadata::new(
            value.tile_size,
            value.binary_type,
            value.data_type,
            value.compression,
        )
    }
}

pub fn infer_rasterarrow_schema(
    existing_schema: &Schema,
    raquet_metadata: &RaquetMetadata,
) -> RasterArrowResult<SchemaRef> {
    let mut new_fields: Vec<FieldRef> = Vec::with_capacity(existing_schema.fields().len());
    for existing_field in existing_schema.fields() {
        if let Some(column_meta) = raquet_metadata.columns.get(existing_field.name()) {
            new_fields.push(infer_target_field(existing_field, column_meta)?)
        } else {
            new_fields.push(existing_field.clone());
        }
    }

    Ok(Arc::new(Schema::new_with_metadata(
        new_fields,
        existing_schema.metadata().clone(),
    )))
}

fn infer_target_field(
    existing_field: &Field,
    column_meta: &RaquetColumnMetadata,
) -> RasterArrowResult<FieldRef> {
    let metadata = Arc::new(Metadata::from(column_meta.clone()));

    let target_geo_data_type = RasterArrowType::Raster(RasterType::new(metadata));

    Ok(Arc::new(target_geo_data_type.to_field(
        existing_field.name(),
        existing_field.is_nullable(),
    )))
}
