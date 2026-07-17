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
    async fn test_quadbin_from_lonlat() {
        let ctx = get_ctx().await;

        let sql =
            r#"select quadbin_from_lonlat(-73.8226318359375,40.60144147645397,9) as lonlat ;"#;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_quadbin_from_tile() {
        let ctx = get_ctx().await;

        let sql = r#"select quadbin_from_tile(0, 0, 0) as quadbin ;"#;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_quadbin_kring() {
        let ctx = get_ctx().await;

        let sql =
            r#"SELECT quadbin_kring(cast(5256690695657226239 as bigint unsigned),1) as kring ;"#;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_quadbin_resolution() {
        let ctx = get_ctx().await;

        let sql = r#"SELECT quadbin_resolution(cast(5256690695657226239 as bigint unsigned)) as resolution ;"#;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_quadbin_sibling() {
        let ctx = get_ctx().await;

        let sql =
            r#"SELECT quadbin_sibling(cast(5256690695657226239 as bigint unsigned)) as sibling ;"#;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_quadbin_to_bbox_mercator() {
        let ctx = get_ctx().await;

        let sql = r#"SELECT quadbin_to_bbox_mercator(cast(5256690695657226239 as bigint unsigned)) as bbox_mercator ;"#;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_quadbin_to_bbox_wgs84() {
        let ctx = get_ctx().await;

        let sql = r#"SELECT quadbin_to_bbox_wgs84(cast(5256690695657226239 as bigint unsigned)) as bbox_wgs84 ;"#;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_quadbin_to_bbox() {
        let ctx = get_ctx().await;

        let sql =
            r#"SELECT quadbin_to_bbox(cast(5256690695657226239 as bigint unsigned)) as bbox ;"#;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_quadbin_to_children() {
        let ctx = get_ctx().await;

        let sql = r#"SELECT quadbin_to_children(cast(5256690695657226239 as bigint unsigned)) as children ;"#;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_quadbin_to_geojson() {
        let ctx = get_ctx().await;

        let sql = r#"SELECT quadbin_to_geojson(cast(5256690695657226239 as bigint unsigned)) as geojson ;"#;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_quadbin_to_lonlat() {
        let ctx = get_ctx().await;

        let sql =
            r#"SELECT quadbin_to_lonlat(cast(5256690695657226239 as bigint unsigned)) as lonlat ;"#;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
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
