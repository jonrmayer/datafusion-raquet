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

pub async fn test_quadbin_to_tile_1000(ctx: SessionContext) -> Vec<arrow_array::RecordBatch> {
    let sql = r###"

 with test as 
    (
   select
    block
    FROM 
    solar
    where block<>0
     order by block asc
    limit 1000
    )
   select quadbin_to_tile(cast(block as bigint)) as tile from test  ;"###;

    let df = ctx.sql(sql).await.unwrap();
    df.collect().await.unwrap()
}

pub async fn test_quadbin_to_bbox_1000(ctx: SessionContext) -> Vec<arrow_array::RecordBatch> {
    let sql = r###"

 with test as 
    (
    select
    block
    FROM 
    solar
    where block<>0
    order by block asc
    limit 1000
    )
    select quadbin_to_bbox(cast (block as bigint)) as bbox from test ;"###;

    let df = ctx.sql(sql).await.unwrap();
    df.collect().await.unwrap()
}

pub async fn test_quadbin_pixel_xy_1000(ctx: SessionContext) -> Vec<arrow_array::RecordBatch> {
    let sql = r###"

   with test as 
    (
    select
    block
    FROM 
    solar
    where block<>0
    order by block asc
    limit 1000
    ),
    result as 
    (
    select quadbin_pixel_xy(0.0, 0.0, 9, 256) as tile from test  
    )
   select * from result ;"###;

    let df = ctx.sql(sql).await.unwrap();
    df.collect().await.unwrap()
}
#[tokio::main]
async fn main() {
    let ctx = setup().await;
    let (out, duration) = time_async(test_quadbin_to_tile_1000(ctx.clone())).await;
    println!("{:?} {:?}", out.len(), duration);
    let (out, duration) = time_async(test_quadbin_pixel_xy_1000(ctx.clone())).await;
    println!("{:?} {:?}", out.len(), duration);
    let (out, duration) = time_async(test_quadbin_to_bbox_1000(ctx.clone())).await;
    println!("{:?} {:?}", out.len(), duration);
}
