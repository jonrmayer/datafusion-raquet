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

pub async fn test_read_raquet_at() -> Vec<arrow_array::RecordBatch> {
    let ctx = setup().await;
    
    let sql = r###"

    --select quadbin_pixel_xy(-3.7038, 40.4168, 9, 256) as output;

    select raquet_value(cast(block as bigint),band_1,'POINT(-3.7038 40.4168)') p from read_raquet_at('solar','POINT(-3.7038 40.4168)');
    "###;

    let df = ctx.sql(sql).await.unwrap();
    df.clone().show().await.unwrap();
    df.collect().await.unwrap()
}
#[tokio::main]
async fn main() {
    let (out, duration) = time_async(test_read_raquet_at()).await;
    println!("{:?} {:?}",out.len(), duration);
   
}
