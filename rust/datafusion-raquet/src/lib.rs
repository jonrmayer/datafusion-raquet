pub mod error;

pub mod format;
pub mod metadata;
pub mod tables;
pub mod udf;
pub mod views;

pub use format::source::RaquetSource;

pub use metadata::RaquetMetadataReader;

pub use tables::raquet::RaquetTable;
// pub use views::read_raquet_metadata;

// pub use udf::raster::{NativeTile, StatisticsTile};

// #[cfg(test)]
// mod table_provider_tests {

//     use std::sync::Arc;

//     use datafusion::catalog::TableProvider;
//     use datafusion::prelude::{SessionConfig, SessionContext};

//     // use crate::views::read_raquet::read_raquet;
//     // use crate::views::read_raquet_metadata::read_raquet_metadata;

//     use super::*;
//     #[tokio::test]
//     async fn read_data_test() {
//         let path =
//             "/home/jonrm/projects/git/raquet-datafusion/data/parquet/tci_interleaved_gzip.parquet"
//                 .to_string();

//         let ctx =
//             SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));

//         ctx.register_udf(StatisticsTile::default().into());
//         ctx.register_udf(NativeTile::default().into());

//         // ctx.register_udtf(
//         //     "read_raquet",
//         //     Arc::new(ReadRaquet::new())
//         //     // read_raquet(
//         //     //     Arc::clone(ctx.state().catalog_list()),
//         //     //     ctx.state().config_options(),
//         //     // ),
//         // );

//         //  ctx.register_udtf(
//         //     "read_raquet_metadata",
//         //     read_raquet_metadata(
//         //         Arc::clone(ctx.state().catalog_list()),
//         //         ctx.state().config_options(),
//         //     ),
//         // );

//         let t = RaquetTable::from_path(path).await;

//         let _ = ctx.register_table("solar", Arc::new(t));

//         let solar = ctx.table_provider("solar").await.unwrap();
//         // // let solar_raquet = solar.as_any()
//         // //     .downcast_ref::<RaquetTable>()
//         // //     .unwrap();
//         // // solar_raquet.table_config().get_table_url().get_store_location()
//         let solar_schema = solar.schema();
//         println!("{:?}", solar_schema);
//         // ctx.table(table_ref)

//         // ctx.state().catalog_list().

//         // let sql = "select native_tile(band_1) from solar where block<>0  ;";
//         // // let sql = "select count(*) from solar;";

//         // let df = ctx.sql(sql).await.unwrap();
//         // println!("{:?}",df.count().await);
//         // df.show().await.unwrap();
//     }
// }

pub fn register(session_context: &datafusion::prelude::SessionContext) {
    crate::udf::quadbin::register(session_context);

    crate::udf::raster::register(session_context);

    crate::views::register(session_context);

   
}
