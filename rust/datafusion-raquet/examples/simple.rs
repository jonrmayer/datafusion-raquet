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

pub async fn test_simple_1m() -> Vec<arrow_array::RecordBatch> {
    let ctx = setup().await;
    
    let sql = r###"
    --create table test_simple as 
    with 
    test as (
    select * from generate_series(1,100000000) as t1(output)
    ),
    result as 
    (
    select test.output,quadbin_pixel_xy(0.0, 0.0, 4, 256) pixel_xy from test
    ) 
    select * from result ;"###;

    let df = ctx.sql(sql).await.unwrap();
    df.collect().await.unwrap()
}
#[tokio::main]
async fn main() {
    let (out, duration) = time_async(test_simple_1m()).await;
    println!("{:?} {:?}",out.len(), duration);
   
}
