use datafusion::prelude::*;
use datafusion_raquet::*;
use datafusion::execution::object_store::ObjectStoreUrl;
use object_store::ClientOptions;
use object_store::http::{HttpBuilder, HttpStore};
use std::sync::Arc;
use tokio::time::{Duration, Instant};
use url::Url;

pub async fn time_async<F, O>(f: F) -> (O, Duration)
where
    F: Future<Output = O>,
{
    let start = Instant::now();
    let out = f.await;
    let duration = start.elapsed();
    (out, duration)
}

pub async fn setup_remote() -> SessionContext {
    let path =
        "https://storage.googleapis.com/raquet_demo_data/spain_solar_ghi.parquet".to_string();
    let url = Url::parse("https://storage.googleapis.com").unwrap();
    let options = ClientOptions::new().with_allow_http(true);
    let object_store_url = ObjectStoreUrl::parse(url.origin().ascii_serialization()).unwrap();
    //  let path = "file:///home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet".to_string();
    let storage_container = HttpBuilder::new()
        .with_url(object_store_url.as_str())
        .with_client_options(options)
        .build()
        .unwrap();

    let mut ctx =
        SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

    ctx.runtime_env()
        .register_object_store(&url, Arc::new(storage_container));

    datafusion_raquet::register(&mut ctx);

    let t = RaquetTable::from_path(path).await;

    let _ = ctx.register_table("solar", Arc::new(t));
    ctx
}

pub async fn setup_local() -> SessionContext {
    // let path = "https://storage.googleapis.com/raquet_demo_data/spain_solar_ghi.parquet"
    //     .to_string();

     let path = "file:///home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet".to_string();

    let mut ctx =
        SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

    datafusion_raquet::register(&mut ctx);

    let t = RaquetTable::from_path(path).await;

    let _ = ctx.register_table("solar", Arc::new(t));
    ctx
}

pub async fn test_value_read_raquet_at(ctx:SessionContext) -> Vec<arrow_array::RecordBatch> {
    
    let sql = r###"    
    select
   
    raquet_value(cast(block as bigint),band_1,'POINT(-3.7038 40.4168)') value
    
    from 
    read_raquet_at('solar','POINT(-3.7038 40.4168)');
    "###;

    let df = ctx.sql(sql).await.unwrap();
    // df.clone().show().await.unwrap();
    df.collect().await.unwrap()
}

pub async fn test_read_raquet_at(ctx:SessionContext) -> Vec<arrow_array::RecordBatch> {
    
    let sql = r###"    
    select raquet_value(block,band_1,'POINT(-3.7038 40.4168)') val from read_raquet_at('solar','POINT(-3.7038 40.4168)')
    "###;

    let df = ctx.sql(sql).await.unwrap();
    // df.clone().show().await.unwrap();
    df.collect().await.unwrap()
}

pub async fn test_multiple_read_raquet_at(ctx:SessionContext) -> Vec<arrow_array::RecordBatch> {
   
    
    let sql = r###"

   with 
    test as (
    select * from generate_series(1,10) as t1(output)
    ),
    result as 
    (
    select 
    --test.output,
    rr.block
    from 
    read_raquet_at('solar','POINT(-3.7038 40.4168)') rr,test
    ) 
    select * from result
    "###;

    let df = ctx.sql(sql).await.unwrap();
    // df.clone().show().await.unwrap();
    df.collect().await.unwrap()
}
#[tokio::main]
async fn main() {
    let (ctx, duration) = time_async(setup_local()).await;
    println!("setup {:?}",  duration);
    let (out, duration) = time_async(test_read_raquet_at(ctx.clone())).await;
    println!("single {:?} {:?}",out[0].num_rows(), duration);
    //  let (out, duration) = time_async(test_value_read_raquet_at(ctx.clone())).await;
    // println!("single value {:?} {:?}",out[0].num_rows(), duration);
    // let (out, duration) = time_async(test_multiple_read_raquet_at(ctx)).await;
    // println!("multiple {:?} {:?}",out[0].num_rows(), duration);
   
}
