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

use rastertile_rs::{BinaryType, CompressionFormat, DataType as RasterDataType};

use parquet::errors::Result;

use arrow_array::cast::as_string_array;
use futures::TryStreamExt;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use arrow_schema::{Field, FieldRef, Schema, SchemaRef};

use rastertile_schema::error::RasterArrowResult;
use rastertile_schema::{Metadata as TileMetadata, RasterArrowType, RasterType};

use rastertile_rs::Metadata as RasterTileMetadata;

use quadbin_schema::{Metadata as QMetadata, QuadbinArrowType, QuadbinType};

pub fn raquet_format_from_str(raquet_str: &str) -> RaquetFormat {
    let raquet_format: RaquetFormat = serde_json::from_str(raquet_str).unwrap();
    raquet_format
}

pub fn raquet_quadbin_metadata(raquet_str: &str) -> QuadbinColumnMetadata {
    let raquet_format = raquet_format_from_str(raquet_str);
    let min_zoom = raquet_format.tiling().unwrap().min_zoom.unwrap();
    let max_zoom = raquet_format.tiling().unwrap().max_zoom.unwrap();
    QuadbinColumnMetadata::new(min_zoom, max_zoom)
}

pub fn raquet_band_metadata(band_name: &str, raquet_str: &str) -> RasterTileMetadata {
    let raquet_format = raquet_format_from_str(raquet_str);
    let tile_size = raquet_format.tiling().unwrap().block_height.unwrap() as usize;

    let compression_str = raquet_format.compression().unwrap();
    let compression = CompressionFormat::from_str(&compression_str).unwrap();
    let bands = raquet_format.bands().unwrap();
    let band: Vec<_> = bands
        .iter()
        .filter(|&x| x.clone().name.unwrap() == band_name)
        .collect();
    let no_data = raquet_format.get_no_data();
    let dtype = band[0].r#type.clone().unwrap();
    let rdt = RasterDataType::from_str(&dtype).unwrap();

    //     tile_size,
    //     rdt,
    //     no_data.clone(),
    //     compression,
    //     BinaryType::Separated,
    //     None,
    // );
    RasterTileMetadata {
        tile_size,
        binary_type: BinaryType::Separated,
        data_type: rdt,
        no_data,
        compression,
        bands: None,
    }
}

pub fn get_quadbin_metadata(
    raquet_format: RaquetFormat,
    existing_schema: Schema,
) -> Result<QuadbinMetadata> {
    let mut columns: HashMap<String, QuadbinColumnMetadata> = HashMap::new();
    let version = raquet_format.version().unwrap();
    let min_zoom = raquet_format.tiling().unwrap().min_zoom.unwrap();
    let max_zoom = raquet_format.tiling().unwrap().max_zoom.unwrap();
    let quadbin_column_name = existing_schema
        .fields()
        .iter()
        .filter(|&x| *x.data_type() == arrow::datatypes::DataType::UInt64)
        .collect::<Vec<_>>()[0]
        .name();
    let qcm = QuadbinColumnMetadata::new(min_zoom, max_zoom);
    columns.insert(quadbin_column_name.to_string(), qcm);

    let qm = QuadbinMetadata { version, columns };

    Ok(qm)
}

pub fn get_raquet_metadata(
    raquet_format: RaquetFormat,
    existing_schema: Schema,
) -> Result<RaquetMetadata> {
    let mut columns: HashMap<String, RaquetColumnMetadata> = HashMap::new();

    let version = raquet_format.version().unwrap();
    let tile_size = raquet_format.tiling().unwrap().block_height.unwrap() as usize;

    let compression_str = raquet_format.compression().unwrap();
    let compression = CompressionFormat::from_str(&compression_str).unwrap();

    let raster_columns = existing_schema
        .fields()
        .iter()
        .filter(|&x| *x.data_type() == arrow::datatypes::DataType::Binary)
        .collect::<Vec<_>>();
    let bands = raquet_format.bands().unwrap();
    let no_data = raquet_format.get_no_data();

    if bands.len() == raster_columns.len() {
        let binary_type: BinaryType = BinaryType::Separated;
        for band in bands.iter() {
            let name = band.name.clone().unwrap();
            let dtype = band.r#type.clone().unwrap();
            let rdt = RasterDataType::from_str(&dtype).unwrap();
            let rcm = RaquetColumnMetadata::new(
                tile_size,
                rdt,
                no_data.clone(),
                compression,
                binary_type,
                None,
            );
            columns.insert(name, rcm);
        }
    } else {
        let binary_type: BinaryType = BinaryType::Interleaved;
        let band = &bands[0];
        let names: Vec<String> = bands.iter().map(|x| x.clone().name.unwrap()).collect();
        let name = raster_columns[0].name().clone();
        let dtype = band.r#type.clone().unwrap();
        let rdt = RasterDataType::from_str(&dtype).unwrap();
        let rcm = RaquetColumnMetadata::new(
            tile_size,
            rdt,
            no_data,
            compression,
            binary_type,
            Some(names),
        );
        columns.insert(name, rcm);
    }

    let rm = RaquetMetadata { version, columns };

    Ok(rm)
}

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
        let raquet_metadata =
            get_raquet_metadata(raquet_format.clone(), existing_schema.clone()).unwrap();
        let quadbin_metadata =
            get_quadbin_metadata(raquet_format.clone(), existing_schema.clone()).unwrap();

        infer_rastertile_schema(&existing_schema, &raquet_metadata, &quadbin_metadata).unwrap()
    }

    // fn get_quadbin_metadata(
    //     &self,
    //     raquet_format: RaquetFormat,
    //     existing_schema: Schema,
    // ) -> Result<QuadbinMetadata> {
    //     let mut columns: HashMap<String, QuadbinColumnMetadata> = HashMap::new();
    //     let version = raquet_format.version().unwrap();
    //     let min_zoom = raquet_format.tiling().unwrap().min_zoom.unwrap();
    //     let max_zoom = raquet_format.tiling().unwrap().max_zoom.unwrap();
    //     let quadbin_column_name = existing_schema
    //         .fields()
    //         .iter()
    //         .filter(|&x| *x.data_type() == arrow::datatypes::DataType::UInt64)
    //         .collect::<Vec<_>>()[0]
    //         .name();
    //     let qcm = QuadbinColumnMetadata::new(min_zoom, max_zoom);
    //     columns.insert(quadbin_column_name.to_string(), qcm);

    //     let qm = QuadbinMetadata { version, columns };

    //     return Ok(qm);
    // }

    // fn get_raquet_metadata(
    //     &self,
    //     raquet_format: RaquetFormat,
    //     existing_schema: Schema,
    // ) -> Result<RaquetMetadata> {
    //     let mut columns: HashMap<String, RaquetColumnMetadata> = HashMap::new();

    //     let version = raquet_format.version().unwrap();
    //     let tile_size = raquet_format.tiling().unwrap().block_height.unwrap() as usize;

    //     let compression_str = raquet_format.compression().unwrap();
    //     let compression = CompressionFormat::from_str(&compression_str).unwrap();

    //     let raster_columns = existing_schema
    //         .fields()
    //         .iter()
    //         .filter(|&x| *x.data_type() == arrow::datatypes::DataType::Binary)
    //         .collect::<Vec<_>>();
    //     let bands = raquet_format.bands().unwrap();
    //     let no_data = raquet_format.get_no_data();

    //     if bands.len() == raster_columns.len() {
    //         let binary_type: BinaryType = BinaryType::Separated;
    //         for (_, band) in bands.iter().enumerate() {
    //             let name = band.name.clone().unwrap();
    //             let dtype = band.r#type.clone().unwrap();
    //             let rdt = RasterDataType::from_str(&dtype).unwrap();
    //             let rcm = RaquetColumnMetadata::new(
    //                 tile_size,
    //                 rdt,
    //                 no_data.clone(),
    //                 compression,
    //                 binary_type,
    //                 None,
    //             );
    //             columns.insert(name, rcm);
    //         }
    //     } else {
    //         let binary_type: BinaryType = BinaryType::Interleaved;
    //         let band = &bands[0];
    //         let names: Vec<String> = bands.iter().map(|x| x.clone().name.unwrap()).collect();
    //         let name = raster_columns[0].name().clone();
    //         let dtype = band.r#type.clone().unwrap();
    //         let rdt = RasterDataType::from_str(&dtype).unwrap();
    //         let rcm = RaquetColumnMetadata::new(
    //             tile_size,
    //             rdt,
    //             no_data,
    //             compression,
    //             binary_type,
    //             Some(names),
    //         );
    //         columns.insert(name, rcm);
    //     }

    //     let rm = RaquetMetadata { version, columns };

    //     return Ok(rm);
    // }

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
pub struct QuadbinMetadata {
    /// The version identifier for the GeoParquet specification.
    pub version: String,

    /// Metadata about geometry columns. Each key is the name of a geometry column in the table.
    pub columns: HashMap<String, QuadbinColumnMetadata>,
}

/// Raquet column metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuadbinColumnMetadata {
    pub min_zoom: i32,
    pub max_zoom: i32,
}

impl QuadbinColumnMetadata {
    pub fn new(min_zoom: i32, max_zoom: i32) -> Self {
        QuadbinColumnMetadata { min_zoom, max_zoom }
    }
}

// impl From<QuadbinColumnMetadata> for TileMetadata {
//     fn from(value: QuadbinColumnMetadata) -> Self {
//         TileMetadata::new(
//             value.tile_size,
//             value.binary_type,
//             value.data_type,
//             value.compression,
//         )
//     }
// }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RaquetMetadata {
    /// The version identifier for the GeoParquet specification.
    pub version: String,

    /// Metadata about geometry columns. Each key is the name of a geometry column in the table.
    pub columns: HashMap<String, RaquetColumnMetadata>,
}

/// Raquet column metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RaquetColumnMetadata {
    pub tile_size: usize,
    pub binary_type: BinaryType,
    pub data_type: RasterDataType,
    pub no_data: String,
    pub compression: CompressionFormat,
    pub bands: Option<Vec<String>>,
}

impl RaquetColumnMetadata {
    pub fn new(
        tile_size: usize,
        data_type: RasterDataType,
        no_data: String,
        compression: CompressionFormat,
        binary_type: BinaryType,
        bands: Option<Vec<String>>,
    ) -> Self {
        RaquetColumnMetadata {
            tile_size,
            binary_type,
            data_type,
            no_data,
            compression,
            bands,
        }
    }
}

impl From<RaquetColumnMetadata> for TileMetadata {
    fn from(value: RaquetColumnMetadata) -> Self {
        TileMetadata::new(
            value.tile_size,
            value.binary_type,
            value.data_type,
            value.no_data,
            value.compression,
            value.bands,
        )
    }
}

impl From<QuadbinColumnMetadata> for QMetadata {
    fn from(value: QuadbinColumnMetadata) -> Self {
        QMetadata::new(value.min_zoom, value.max_zoom)
    }
}

pub fn infer_rastertile_schema(
    existing_schema: &Schema,
    raquet_metadata: &RaquetMetadata,
    quadbin_metadata: &QuadbinMetadata,
) -> RasterArrowResult<SchemaRef> {
    let mut new_fields: Vec<FieldRef> = Vec::with_capacity(existing_schema.fields().len());
    for existing_field in existing_schema.fields() {
        if let Some(column_meta) = raquet_metadata.columns.get(existing_field.name()) {
            new_fields.push(infer_target_field(existing_field, column_meta)?)
        } else if let Some(column_meta) = quadbin_metadata.columns.get(existing_field.name()) {
            new_fields.push(infer_quadbin_field(existing_field, column_meta)?)
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
    let metadata = Arc::new(TileMetadata::from(column_meta.clone()));

    let target_geo_data_type = RasterArrowType::Raster(RasterType::new(metadata));

    Ok(Arc::new(target_geo_data_type.to_field(
        existing_field.name(),
        existing_field.is_nullable(),
    )))
}

fn infer_quadbin_field(
    existing_field: &Field,
    column_meta: &QuadbinColumnMetadata,
) -> RasterArrowResult<FieldRef> {
    let metadata = Arc::new(QMetadata::from(column_meta.clone()));

    let target_quadbin_data_type = QuadbinArrowType::QuadbinU64(QuadbinType::new(metadata));

    Ok(Arc::new(target_quadbin_data_type.to_field(
        existing_field.name(),
        existing_field.is_nullable(),
    )))
}
