#[cfg(test)]
mod tests {

    use crate::udtf::register;
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
    async fn test_intersecting_cells_tbl() {
        let ctx = get_ctx().await;
        let sql = r###"
        with tbl as (
            SELECT count(quadbin) FROM intersecting_cells_tbl('POLYGON((-45 40, 0 40, 0 66, -45 66, -45 40))', 10)
        )

        select * from tbl
       
       

        "###;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_boundary_cells_tbl() {
        let ctx = get_ctx().await;
        let sql = r###"
        with tbl as (
            SELECT count(quadbin) FROM boundary_cells_tbl('POLYGON((-45 40, 0 40, 0 66, -45 66, -45 40))', 10)
        )

        select * from tbl
       
       

        "###;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_contained_cells_tbl() {
        let ctx = get_ctx().await;
        let sql = r###"
        with tbl as (
            SELECT count(quadbin) FROM contained_cells_tbl('POLYGON((-45 40, 0 40, 0 66, -45 66, -45 40))', 10)
        )

        select * from tbl
       
       

        "###;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }
}

// SELECT CAST([1, 2, 3, 4] AS TINYINT[]) AS i8_array;
