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

pub async fn setup() -> SessionContext {
    let path = "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet"
        .to_string();

    let mut ctx =
        SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

    datafusion_raquet::register(&mut ctx);

    let t = RaquetTable::from_path(path).await;

    let _ = ctx.register_table("solar", Arc::new(t));
    ctx
}

pub async fn test_simple_stats() -> Vec<arrow_array::RecordBatch> {
    let ctx = setup().await;

    let sql = r###"
    with data as (
    SELECT block,statistics_tile(band_1) as stats from solar where block<>0    
   
    ),
     out as (select block,unnest(stats) l from data)

    select * from out
    ;"###;

    let df = ctx.sql(sql).await.unwrap();
    df.collect().await.unwrap()
}

pub async fn test_complex_stats() -> Vec<arrow_array::RecordBatch> {
    let ctx = setup().await;

    let sql = r###"
    with data as (
    SELECT block,native_tile(band_1) as native from solar where block<>0 
    limit 100
    ),
    out as ( select block,array_length(native,1) l from data)

    select l total_pixels from out
    ;"###;

    let df = ctx.sql(sql).await.unwrap();
    df.collect().await.unwrap()
}
#[tokio::main]
async fn main() {
    let (out, duration) = time_async(test_simple_stats()).await;
    println!("{:?} {:?} {:?}","test_simple_stats", out.len(), duration);

    // let (out, duration) = time_async(test_complex_stats()).await;
    // println!("{:?} {:?} {:?}", "test_complex_stats",out.len(), duration);
}
