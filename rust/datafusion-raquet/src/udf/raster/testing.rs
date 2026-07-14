#[cfg(test)]
mod tests {

    use crate::udf::general::register as general_register;
    use crate::udf::raster::register as raster_register;
    use datafusion::prelude::{SessionConfig, SessionContext};

    pub async fn get_ctx() -> SessionContext {
        let ctx =
            SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));
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

    #[tokio::test]
    async fn test_statistics_tile() {
        let ctx = get_ctx().await;
        //  let sql = r#"SELECT statistics_tile(band_1) as stats from solar where block=5230520127799164927  ;"#;

        let sql = r#"SELECT raquet_band_statistics(cast_raquet(band_1,'256', 'Separated', 'float32','NaN','gzip')) as stats from solar where block=5230520127799164927  ;"#;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_native_tile() {
        let ctx = get_ctx().await;
        let sql = r#"SELECT native_tile(cast_raquet(band_1,'256', 'Separated', 'float32','NaN','gzip')) as native from solar where block=5230520127799164927  ;"#;
        let df = ctx.sql(sql).await.unwrap();
        // df.show().await.unwrap();
    }
}
