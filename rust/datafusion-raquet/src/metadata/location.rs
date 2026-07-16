use std::sync::Arc;

use parquet::arrow::ProjectionMask;
use parquet::arrow::arrow_reader::RowFilter;
use parquet::file::metadata::ParquetMetaData;

use crate::metadata::metadata_arrow_predicate;

#[derive(Clone, Debug)]
pub struct MetaDataLocation {
    pub metadata: Arc<ParquetMetaData>,
}

impl MetaDataLocation {
    pub fn new(metadata: Arc<ParquetMetaData>) -> Self {
        MetaDataLocation { metadata }
    }

    pub fn metadata(&self) -> Arc<ParquetMetaData> {
        self.metadata.clone()
    }

    pub fn column_index(&self) -> usize {
        
        self
            .metadata()
            .file_metadata()
            .schema()
            .get_fields()
            .iter()
            .position(|r| r.name() == "metadata")
            .unwrap()
    }

    pub fn row_group_indexes(&self) -> Vec<usize> {
        let mut row_group_indexes: Vec<usize> = Vec::new();
        for (i, r) in self.metadata().row_groups().iter().enumerate() {
            if r.num_rows() == 1 {
                row_group_indexes.push(i);
            }
        }
        row_group_indexes
    }

    pub fn row_filter(&self) -> RowFilter {
        let predicate =
            metadata_arrow_predicate(&self.metadata().file_metadata().schema_descr_ptr()).unwrap();
        RowFilter::new(vec![predicate])
    }

    pub fn projection(&self) -> ProjectionMask {
        

        ProjectionMask::roots(
            &self.metadata().file_metadata().schema_descr_ptr(),
            vec![self.column_index()],
        )
    }
}
