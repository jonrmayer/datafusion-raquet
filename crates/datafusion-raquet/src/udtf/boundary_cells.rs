use std::any::Any;
use std::sync::Arc;

use arrow_array::cast::{as_primitive_array, as_string_array};
use arrow_array::{ArrayRef, Int64Array, RecordBatch, UInt64Array};
use arrow_schema::{DataType, Field, Schema, SchemaRef};
use quadbin_geo_rs::GeoCells;

use datafusion::catalog::{Session, TableFunctionImpl, TableProvider};
use datafusion::datasource::TableType;
use datafusion::datasource::memory::MemorySourceConfig; // <-- NEW
use datafusion::datasource::source::DataSourceExec;
use datafusion::error::{DataFusionError, Result};
use datafusion::execution::context::SessionState;
use datafusion::logical_expr::Expr;
use datafusion::physical_plan::ExecutionPlan;
use datafusion_common::DFSchema;


use datafusion::physical_plan::PhysicalExpr;
use datafusion::physical_plan::expressions::Column;
use datafusion::physical_plan::projection::ProjectionExec;

/// Dynamic TableProvider mapping logic
#[derive(Debug)]
pub struct BoundaryCellsTableProvider {
    wkt_expr: Expr,
    resolution_expr: Expr,
    schema: SchemaRef,
}

impl BoundaryCellsTableProvider {
    pub fn new(wkt_expr: Expr, resolution_expr: Expr) -> Self {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "quadbin",
            DataType::UInt64,
            true,
        )]));
        Self {
            wkt_expr,
            resolution_expr,
            schema,
        }
    }
}

#[async_trait::async_trait]
impl TableProvider for BoundaryCellsTableProvider {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    async fn scan(
        &self,
        state: &dyn Session, // Trait object signature standard
        projection: Option<&Vec<usize>>,
        _filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        // 1. Safely retrieve the concrete SessionState using the downcast pattern
        let session_state = state
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| {
                DataFusionError::Internal("Failed to downcast SessionState".to_string())
            })?;

        let empty_batch = RecordBatch::new_empty(Arc::new(Schema::empty()));
        let empty_df_schema = DFSchema::empty();

        // 2. Map logical variables out into physical expression targets
        let wkt_phys =
            session_state.create_physical_expr(self.wkt_expr.clone(), &empty_df_schema)?;
        let res_phys =
            session_state.create_physical_expr(self.resolution_expr.clone(), &empty_df_schema)?;

        let wkt_val = wkt_phys.evaluate(&empty_batch)?;
        let res_val = res_phys.evaluate(&empty_batch)?;

        let wkt_array = wkt_val.into_array(1)?;
        let resolution_array = res_val.into_array(1)?;

        let wkt_str = as_string_array(&wkt_array);
        let res_int: &Int64Array = as_primitive_array(&resolution_array);

        let mut all_geocells: Vec<u64> = Vec::new();

        for (wkt, resolution) in wkt_str.iter().zip(res_int.iter()) {
            if let (Some(w_val), Some(r_val)) = (wkt, resolution) {
                let geocells = GeoCells::new(w_val.to_string(), r_val as i8)
                    .boundary_cells()
                    .map_err(|e| DataFusionError::Execution(e.to_string()))?;

                all_geocells.extend(geocells);
            }
        }

        if let Some(l) = limit {
            all_geocells.truncate(l);
        }

        let quadbin_array = Arc::new(UInt64Array::from(all_geocells)) as ArrayRef;
        let batch = RecordBatch::try_new(Arc::clone(&self.schema), vec![quadbin_array])?;

        let mem_config =
            MemorySourceConfig::try_new(&[vec![batch]], Arc::clone(&self.schema), None)?;

        let base_exec = DataSourceExec::new(Arc::new(mem_config));
        let mut exec_plan: Arc<dyn ExecutionPlan> = Arc::new(base_exec);

        // 💥 FIX: Eliminate ProjectionExprs structure. Construct a clean layout vector of
        // mapped tuples directly compatible with the physical ProjectionExec builder interface instead.
        if let Some(proj) = projection {
            if !proj.is_empty() {
                let current_schema = exec_plan.schema();
                let mut projection_exprs = Vec::new();

                for &index in proj {
                    let field = current_schema.field(index);
                    // Instantiate standard column index mapper assignments
                    let phys_col =
                        Arc::new(Column::new(field.name(), index)) as Arc<dyn PhysicalExpr>;

                    // Build vector tuples compatible with DataFusion physical plan interfaces
                    projection_exprs.push((phys_col, field.name().to_string()));
                }

                // Feed the vector configuration straight into the ExecutionPlan constructor wrapper
                exec_plan = Arc::new(ProjectionExec::try_new(projection_exprs, exec_plan)?);
            }
        }

        Ok(exec_plan)
    }
}


// =========================================================================
// 2. TableFunctionImpl Hook Mapping
// =========================================================================
#[derive(Debug, Default)]
pub struct BoundaryCellsUDTF;

impl BoundaryCellsUDTF {
    pub fn new() -> Self {
        Self
    }
}

impl TableFunctionImpl for BoundaryCellsUDTF {
    
    fn call(&self, exprs: &[Expr]) -> Result<Arc<dyn TableProvider>> {
        if exprs.len() != 2 {
            return Err(DataFusionError::Plan(
                "quadbin_polyfill requires exactly 2 arguments (wkt: Utf8, resolution: Int64)"
                    .to_string(),
            ));
        }

        // Return the dynamic table source context provider
        Ok(Arc::new(BoundaryCellsTableProvider::new(
            exprs[0].clone(),
            exprs[1].clone(),
        )))
    }
}
