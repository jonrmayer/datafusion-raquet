use std::sync::Arc;

#[cfg(any(test, debug_assertions))]
pub mod testing;

mod intersecting_cells;
mod boundary_cells;
mod contained_cells;

pub use intersecting_cells::IntersectingCellsUDTF;
pub use boundary_cells::BoundaryCellsUDTF;
pub use contained_cells::ContainedCellsUDTF;

pub fn register(session_context: &datafusion::prelude::SessionContext) {
    session_context.register_udtf("intersecting_cells_tbl", Arc::new(IntersectingCellsUDTF::new()));
    session_context.register_udtf("boundary_cells_tbl", Arc::new(BoundaryCellsUDTF::new()));
    session_context.register_udtf("contained_cells_tbl", Arc::new(ContainedCellsUDTF::new()));
    // session_context.register_udf(Intersects::default().into());
    // session_context.register_udf(QuadbinMetadata::default().into());
    // session_context.register_udf(QuadBinToPixelXY::default().into());
    // session_context.register_udf(QuadBinPolyFill::default().into());
    // session_context.register_udf(CastRaquet::default().into());
}