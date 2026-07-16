use datafusion::execution::object_store::ObjectStoreUrl;
use datafusion::prelude::*;
use datafusion_raquet::*;
use object_store::ClientOptions;
use object_store::http::{HttpBuilder, HttpStore};
use std::sync::Arc;
use tokio::time::{Duration, Instant};
use url::Url;

// use datafusion::common::DataFusionError;
// use datafusion_raquet::udf::general::intersects::Intersects;
// use datafusion_raquet::udf::raster::{
//     CastRaquet, DecodeTile, DecompressTile, ParquetDecodeTile, ParquetStatisticsTile, ParquetValue,
// };

pub async fn time_async<F, O>(f: F) -> (O, Duration)
where
    F: Future<Output = O>,
{
    let start = Instant::now();
    let out = f.await;
    let duration = start.elapsed();
    (out, duration)
}

pub async fn setup_local_store() -> SessionContext {
    // let path =
    //     "file:///home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet"
    //         .to_string();

    let mut ctx =
        SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));
    let sql = r###"
        CREATE EXTERNAL TABLE spain_solar
        STORED AS PARQUET
        LOCATION '/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet';
    "###;
    let _ = ctx.sql(sql).await;
    // datafusion_raquet::register(&mut ctx);

    // let t = RaquetTable::from_path(path).await;

    // let _ = ctx.register_table("solar", Arc::new(t));
    ctx
}

// pub async fn setup_local_parquet() -> SessionContext {
//     let path = "/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet";

//     let ctx = SessionContext::new();
//     // SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

//     // register(&mut ctx);
//     ctx.register_udf(DecodeTile::default().into());
//     ctx.register_udf(DecompressTile::default().into());
//     ctx.register_udf(ParquetDecodeTile::default().into());
//     ctx.register_udf(ParquetStatisticsTile::default().into());
//     ctx.register_udf(ParquetValue::default().into());
//     ctx.register_udf(Intersects::default().into());
//     ctx.register_udf(CastRaquet::default().into());
//     let _ = ctx
//         .register_parquet("solar", path, ParquetReadOptions::default())
//         .await
//         .map_err(|e| {
//             DataFusionError::Context(format!("Registering 'hits_raw' as {path}"), Box::new(e))
//         });

//     ctx
// }

// pub async fn test_read_parquet_aaa(ctx: SessionContext) -> Vec<arrow_array::RecordBatch> {
//     let sql = r###"

//      with m as (
//             select metadata from solar where block=0

//         ),
//         data as (
//             select block,band_1  from solar
//             where block<>0
//         ),
//         idata as (
//             select unnest(intersects('POINT(-3.7038 40.4168)',m.metadata)) indata from m
//         )

//         select parquet_value(data.band_1,'POINT(-3.7038 40.4168)',m.metadata) value from data,m,idata

//         where idata.indata=data.block

//     "###;

//     let df = ctx.sql(sql).await.unwrap();
//     df.collect().await.unwrap()
// }
pub async fn test_read_parquet_band(ctx: SessionContext) -> Vec<arrow_array::RecordBatch> {
    let sql = r###"
        select count(*) from spain_solar;
    "###;
    let df = ctx.sql(sql).await.unwrap();
    df.clone().show().await;
    df.collect().await.unwrap()
}
// pub async fn test_read_parquet_cast(ctx: SessionContext) -> Vec<arrow_array::RecordBatch> {
//     let sql = r###"
//        select cast_raquet(band_1,'256', 'Separated', 'float32','NaN','gzip') from solar where block<>0
//     "###;

//     let df = ctx.sql(sql).await.unwrap();
//     df.collect().await.unwrap()
// }

// pub async fn test_read_parquet_cast_decode(ctx: SessionContext) -> Vec<arrow_array::RecordBatch> {
//     let sql = r###"
//     with cast_data as (
// select cast_raquet(band_1,'256', 'Separated', 'float32','NaN','gzip') band_1 from solar where block<>0

//     )

//        select decode_tile(band_1) from cast_data
//         "###;

//     let df = ctx.sql(sql).await.unwrap();
//     df.collect().await.unwrap()
// }

// pub async fn test_read_raquet_band(ctx: SessionContext) -> Vec<arrow_array::RecordBatch> {
//     let sql = r###"

//        select band_1 from solar where block<>0
//     "###;

//     let df = ctx.sql(sql).await.unwrap();
//     df.collect().await.unwrap()
// }

// pub async fn test_read_raquet_decode(ctx: SessionContext) -> Vec<arrow_array::RecordBatch> {
//     let sql = r###"

//        select decode_tile(band_1) from solar where block<>0
//     "###;

//     let df = ctx.sql(sql).await.unwrap();
//     df.collect().await.unwrap()
// }
// async fn test_local_parquet() {
//     let (ctx, duration) = time_async(setup_local_parquet()).await;
//     println!("local parquet setup {:?}", duration);
//     let (out, duration) = time_async(test_read_parquet_band(ctx.clone())).await;
//     println!(
//         "test_read_parquet_band {:?} {:?}",
//         out[0].num_rows(),
//         duration
//     );
//     let (out, duration) = time_async(test_read_parquet_cast(ctx.clone())).await;
//     println!(
//         "test_read_parquet_cast {:?} {:?}",
//         out[0].num_rows(),
//         duration
//     );
//     let (out, duration) = time_async(test_read_parquet_cast_decode(ctx.clone())).await;
//     println!(
//         "test_read_parquet_cast_decode {:?} {:?}",
//         out[0].num_rows(),
//         duration
//     );
//     //  let (out, duration) = time_async(test_read_raquet2(ctx.clone())).await;
//     // println!("local parquet {:?} {:?}", out[0].num_rows(), duration);
// }

// async fn test_local_raquet() {
//     let (ctx, duration) = time_async(setup_local_raquet()).await;
//     println!("local raquet setup {:?}", duration);
//     let (out, duration) = time_async(test_read_raquet_band(ctx.clone())).await;
//     println!(
//         "test_read_raquet_band {:?} {:?}",
//         out[0].num_rows(),
//         duration
//     );
//     let (out, duration) = time_async(test_read_raquet_decode(ctx.clone())).await;
//     println!(
//         "test_read_raquet_decode {:?} {:?}",
//         out[0].num_rows(),
//         duration
//     );
// }

// // async fn test_remote() {
// //     let (ctx, duration) = time_async(setup_remote()).await;
// //     println!("remote setup {:?}", duration);
// //     let (out, duration) = time_async(test_read_raquet(ctx)).await;
// //     println!("remote single {:?} {:?}", out[0].num_rows(), duration);
// // }
#[tokio::main]
async fn main() {
    let (ctx, duration) = time_async(setup_local_store()).await;
    println!("remote setup {:?}", duration);
    let (out, duration) = time_async(test_read_parquet_band(ctx.clone())).await;
    println!("local parquet {:?} {:?}", out[0].num_rows(), duration);

    // test_local_parquet().await;
    // test_local_raquet().await;
}
