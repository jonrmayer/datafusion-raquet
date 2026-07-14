#[cfg(test)]
mod tests {

    use crate::udf::general::register;
    use datafusion::prelude::{SessionConfig, SessionContext};

    pub async fn get_ctx() -> SessionContext {
        let ctx =
            SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));
        register(&ctx);
        let sql = r###"
        CREATE EXTERNAL TABLE solar
        STORED AS PARQUET
        LOCATION '/home/jonrm/projects/git/raquet-datafusion/data/parquet/spain_solar_ghi.parquet';
        "###;
        let _ = ctx.sql(sql).await;
        ctx
    }

    #[tokio::test]
    async fn test_intersects() {
        let ctx = get_ctx().await;
        let sql = r###"
        with m as (
            select metadata from solar where block=0

        ),
        data as (
            select unnest(intersects('POINT(-3.7038 40.4168)',m.metadata)) indata from m
        )

        select solar.block from data,solar
        where data.indata=solar.block
       

   
        "###;

        

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_band_metadata() {
        let ctx = get_ctx().await;
        let sql = r###"
        with m as (
            select metadata from solar where block=0

        )

        select band_metadata('band_1',m.metadata) as band_meta from m

        "###;

        

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_quadbin_metadata() {
        let ctx = get_ctx().await;
        let sql = r###"
        with m as (
            select metadata from solar where block=0

        )

        select quadbin_metadata(m.metadata) as quadbin_meta from m

        "###;

       

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_quadbin_pixel_xy() {
        let ctx = get_ctx().await;
        let sql = r###"
       
        select quadbin_pixel_xy(0.0, 0.0, 9, 256) as pixel 

        "###;

       

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_quadbin_polyfill() {
        let ctx = get_ctx().await;
        let sql = r###"
       
        select quadbin_polyfill('POLYGON((-45 40.9798980696201, 0 40.9798980696201, 0 66.5132604431119, -45 66.5132604431119, -45 40.9798980696201))',4) qlist

        "###;

        
        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_cast_raquet() {
        let ctx = get_ctx().await;
        let sql = r###"
       
         select cast_raquet(band_1,'256', 'Separated', 'float32','NaN','gzip') band_1 from solar where block<>0 limit 1

        "###;

       

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }
}
