pub mod file_format;
pub mod source;

// #[cfg(test)]
// mod tests {
//     use crate::raquet::format::file_format::RaquetFormatFactory;
//     use datafusion::execution::SessionStateBuilder;
//     use datafusion::prelude::SessionContext;
//     use std::sync::Arc;

//     #[tokio::test]
//     async fn test_raquet() {
//         let file_format = Arc::new(RaquetFormatFactory::default());
//         let state = SessionStateBuilder::new()
//             .with_file_formats(vec![file_format])
//             .build();
//         let ctx = SessionContext::new_with_state(state).enable_url_table();

//         let sql = "SELECT block FROM '/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet'  as table limit 1";

//         let df = ctx.sql(sql).await.unwrap();
//         df.show().await.unwrap();

//         let df = ctx.sql(sql).await.unwrap();
//         df.show().await.unwrap();
//     }

//     #[tokio::test]
//     async fn test_raquet_metadata() {
//         let file_format = Arc::new(RaquetFormatFactory::default());
//         let state = SessionStateBuilder::new()
//             .with_file_formats(vec![file_format])
//             .build();
//         let ctx = SessionContext::new_with_state(state).enable_url_table();

//         let sql = "SELECT * FROM '/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet'  as table limit 1";

//         let df = ctx
//             .sql(sql)
//             .await
//             .unwrap();
//         let schema = df.schema();
//         let field = schema.field_with_unqualified_name("band_1").unwrap();
//         println!("{:?}",field);
//         // assert_eq!(field.extension_type_name().unwrap(), "geoarrow.wkb");
//     }
// }
