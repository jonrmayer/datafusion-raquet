use datafusion::prelude::*;
use datafusion_raquet::*;
use std::sync::Arc;
use std::time::SystemTime;
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

pub async fn test_read_raquet() -> Vec<arrow_array::RecordBatch> {
    let ctx = setup().await;
    
    let sql = r###"

    select * from read_raquet('solar');
    "###;

    let df = ctx.sql(sql).await.unwrap();
    df.collect().await.unwrap()
}
#[tokio::main]
async fn main() {
    let (out, duration) = time_async(test_read_raquet()).await;
    println!("{:?} {:?}",out.len(), duration);
   
}
