#[cfg(test)]
mod tests {

    use crate::udf::quadbin::register;
    use datafusion::prelude::{SessionConfig, SessionContext};

    pub async fn get_ctx() -> SessionContext {
        let ctx =
            SessionContext::new_with_config(SessionConfig::new().with_information_schema(true));
        register(&ctx);
        ctx
    }

    #[tokio::test]
    async fn test_quadbin_to_parent() {
        let ctx = get_ctx().await;

        let sql =
            r#"SELECT quadbin_to_parent(cast(5256690695657226239 as bigint unsigned)) as parent ;"#;
       

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_quadbin_to_tile() {
        let ctx = get_ctx().await;

        let sql =
            r#"SELECT quadbin_to_tile(cast(5256690695657226239 as bigint unsigned)) as tile ;"#;
       

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_quadbin_to_wkt() {
        let ctx = get_ctx().await;

        let sql = r#"SELECT quadbin_to_wkt(cast(5256690695657226239 as bigint unsigned)) as wkt ;"#;
      

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }
}
