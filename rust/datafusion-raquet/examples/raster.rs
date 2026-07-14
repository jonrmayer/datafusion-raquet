use crate::udf::general::register as general_register;
use crate::udf::raster::register as raster_register;
use datafusion::prelude::*;
use datafusion_raquet::*;
use std::sync::Arc;
use tokio::time::{Duration, Instant};

pub async fn time_async<F, O>(f: F) -> (O, Duration)
where
    F: Future<Output = O>,
{
    let start = Instant::now();
    let out = f.await;
    let duration = start.elapsed();
    (out, duration)
}

pub async fn get_ctx() -> SessionContext {
    let ctx = SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));
    raster_register(&ctx);
    general_register(&ctx);
    let sql = r###"
        CREATE EXTERNAL TABLE solar
        STORED AS PARQUET
        LOCATION '/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet';
        "###;
    let _ = ctx.sql(sql).await;
    ctx
}

pub async fn test_native_tile_1000(ctx: SessionContext) -> Vec<arrow_array::RecordBatch> {
    let sql = r###"

  with data as (
    SELECT cast_raquet(band_1,'256', 'Separated', 'float32','NaN','gzip') band from solar where block<>0 
    limit 1000
    )

    select native_tile(band) native from data"###;

    let df = ctx.sql(sql).await.unwrap();
    df.collect().await.unwrap()
}

pub async fn test_statistics_tile_1000(ctx: SessionContext) -> Vec<arrow_array::RecordBatch> {
    let sql = r###"

   with data as (
   SELECT cast_raquet(band_1,'256', 'Separated', 'float32','NaN','gzip') band from solar where block<>0 
    limit 1000
   
    )
    select raquet_band_statistics(band) stats from data"###;

    let df = ctx.sql(sql).await.unwrap();
    df.collect().await.unwrap()
}
#[tokio::main]
async fn main() {
    let ctx = get_ctx().await;
    let (out, duration) = time_async(test_native_tile_1000(ctx.clone())).await;
    println!(
        "{:?} {:?} {:?}",
        "test_native_tile_1000",
        out.len(),
        duration
    );
    let (out, duration) = time_async(test_statistics_tile_1000(ctx.clone())).await;
    println!(
        "{:?} {:?} {:?}",
        "test_statistics_tile_1000",
        out.len(),
        duration
    );
}
