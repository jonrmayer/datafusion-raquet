use arrow_array::UInt64Array;

use parquet::arrow::{
    ProjectionMask,
    arrow_reader::{ArrowPredicate, ArrowPredicateFn},
};
use parquet::schema::types::SchemaDescriptor;
use parquet::errors::Result;

pub fn metadata_arrow_predicate(schema_desc: &SchemaDescriptor) -> Result<Box<dyn ArrowPredicate>> {
    let projection = ProjectionMask::leaves(schema_desc, [0]);
    let predicate = ArrowPredicateFn::new(projection, |batch| {
        let block_col = batch.column(0);
        let out = arrow::compute::kernels::cmp::eq(block_col, &UInt64Array::new_scalar(0)).unwrap();
        Ok(out)
    });
    Ok(Box::new(predicate))
}
