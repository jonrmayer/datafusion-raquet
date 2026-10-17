#[cfg(test)]
mod tests {

    use crate::udf::general::register as general_register;
    use crate::udf::raster::register as raster_register;
    use datafusion::prelude::{SessionConfig, SessionContext};

    pub async fn get_ctx() -> SessionContext {
        let mut config = SessionConfig::new().with_information_schema(true);
        // Enable the nested_expressions feature
        // config = config.w
        // with_extension_option("datafusion.optimizer.nested_expressions", true),
        // config = config.set_str("datafusion.optimizer.nested_expressions", "true");

        let ctx = SessionContext::new_with_config(config);
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
        let ctx: SessionContext = get_ctx().await;
        //  let sql = r#"SELECT statistics_tile(band_1) as stats from solar where block=5230520127799164927  ;"#;

        let sql = r#"SELECT raquet_band_statistics(binary_to_raquet(band_1,'256', 'Separated', 'float32','NaN','gzip')) as stats from solar where block=5230520127799164927  ;"#;

        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_native_tile() {
        let ctx = get_ctx().await;
        let sql = r#"SELECT raquet_band_native(binary_to_raquet(band_1,'256', 'Separated', 'float32','NaN','gzip')) as native from solar where block=5230520127799164927  ;"#;
        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_encode_tile() {
        let ctx = get_ctx().await;
        let sql = r###"

      with dt as (
    SELECT 
         CAST([1, 2, 3, 4] as TINYINT[])   AS i8_array,
        CAST([1, 2, 3, 4] as SMALLINT[])  AS i16_array,
        CAST([1, 2, 3, 4] as INT[])       AS i32_array,
        CAST([1, 2, 3, 4] as BIGINT[])    AS i64_array,
        CAST([1, 2, 3, 4] as TINYINT UNSIGNED[])  AS u8_array,
        CAST([1, 2, 3, 4] as SMALLINT UNSIGNED[]) AS u16_array,
        CAST([1, 2, 3, 4] as INT UNSIGNED[])      AS u32_array,
        CAST([1, 2, 3, 4] as BIGINT UNSIGNED[])   AS u64_array,
        CAST([1.0, 2.0, 3.0, 4.0] as FLOAT[])  AS f32_array,
        CAST([1.0, 2.0, 3.0, 4.0] as DOUBLE[]) AS f64_array
),
encode as (
    select 
        raquet_band_encode(dt.i8_array,  '2', 'Separated', 'int8',   'NaN', 'none') as band_i8,
        raquet_band_encode(dt.i16_array, '2', 'Separated', 'int16',  'NaN', 'none') as band_i16,
        raquet_band_encode(dt.i32_array, '2', 'Separated', 'int32',  'NaN', 'none') as band_i32,
        raquet_band_encode(dt.i64_array, '2', 'Separated', 'int64',  'NaN', 'none') as band_i64,
        raquet_band_encode(dt.u8_array,  '2', 'Separated', 'uint8',  'NaN', 'none') as band_u8,
        raquet_band_encode(dt.u16_array, '2', 'Separated', 'uint16', 'NaN', 'none') as band_u16,
        raquet_band_encode(dt.u32_array, '2', 'Separated', 'uint32', 'NaN', 'none') as band_u32,
        raquet_band_encode(dt.u64_array, '2', 'Separated', 'uint64', 'NaN', 'none') as band_u64,
        raquet_band_encode(dt.f32_array, '2', 'Separated', 'float32','NaN', 'none') as band_f32,
        raquet_band_encode(dt.f64_array, '2', 'Separated', 'float64','NaN', 'none') as band_f64
    from dt
),
raquet as (
    select 
        binary_to_raquet(encode.band_i8,  '2', 'Separated', 'int8',   'NaN', 'none') as band_i8,
        binary_to_raquet(encode.band_i16, '2', 'Separated', 'int16',  'NaN', 'none') as band_i16,
        binary_to_raquet(encode.band_i32, '2', 'Separated', 'int32',  'NaN', 'none') as band_i32,
        binary_to_raquet(encode.band_i64, '2', 'Separated', 'int64',  'NaN', 'none') as band_i64,
        binary_to_raquet(encode.band_u8,  '2', 'Separated', 'uint8',  'NaN', 'none') as band_u8,
        binary_to_raquet(encode.band_u16, '2', 'Separated', 'uint16', 'NaN', 'none') as band_u16,
        binary_to_raquet(encode.band_u32, '2', 'Separated', 'uint32', 'NaN', 'none') as band_u32,
        binary_to_raquet(encode.band_u64, '2', 'Separated', 'uint64', 'NaN', 'none') as band_u64,
        binary_to_raquet(encode.band_f32, '2', 'Separated', 'float32','NaN', 'none') as band_f32,
        binary_to_raquet(encode.band_f64, '2', 'Separated', 'float64','NaN', 'none') as band_f64
    from encode
)
select 
    raquet_band_native(raquet.band_i8)  as band_int8,
    raquet_band_native(raquet.band_i16) as band_int16,
    raquet_band_native(raquet.band_i32) as band_int32,
    raquet_band_native(raquet.band_i64) as band_int64,
    raquet_band_native(raquet.band_u8)  as band_uint8,
    raquet_band_native(raquet.band_u16) as band_uint16,
    raquet_band_native(raquet.band_u32) as band_uint32,
    raquet_band_native(raquet.band_u64) as band_uint64,
    raquet_band_native(raquet.band_f32) as band_float32,
    raquet_band_native(raquet.band_f64) as band_float64
from raquet;


        "###;
        let df = ctx.sql(sql).await.unwrap();
        let df_schema = df.schema();

        // 2. Convert to a standard Apache Arrow Schema
        let arrow_schema = df_schema.as_arrow();

        // 3. Inspect columns and data types
        for field in arrow_schema.fields() {
            println!("Column: {}, Type: {:?}", field.name(), field.data_type());

            // Access custom Arrow metadata attached to a specific column/field
            if !field.metadata().is_empty() {
                println!("Field custom metadata: {:?}", field.metadata());
            }
        }
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_decode_tile() {
        let ctx = get_ctx().await;
        let sql = r#"SELECT raquet_band_decode(binary_to_raquet(band_1,'256', 'Separated', 'float32','NaN','gzip')) as native from solar where block=5230520127799164927  ;"#;
        let _df = ctx.sql(sql).await.unwrap();
        // df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_decompress_tile() {
        let ctx = get_ctx().await;
        let sql = r#"SELECT raquet_decompress(binary_to_raquet(band_1,'256', 'Separated', 'float32','NaN','gzip')) as native from solar where block=5230520127799164927  ;"#;
        let _df = ctx.sql(sql).await.unwrap();
        // df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_raquet_value() {
        let ctx = get_ctx().await;
        let sql = r#"SELECT raquet_value(binary_to_raquet(band_1,'256', 'Separated', 'float32','NaN','gzip'),'POINT(-3.7038 40.4168)',cast(9 as INT)) as value from solar where block=5229757908543078399  ;"#;
        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }

    #[tokio::test]
    async fn test_raquet_pixel() {
        let ctx = get_ctx().await;
        let sql = r#"SELECT raquet_pixel(binary_to_raquet(band_1,'256', 'Separated', 'float32','NaN','gzip'),32,17) as value from solar where block=5229757908543078399  ;"#;
        let df = ctx.sql(sql).await.unwrap();
        df.show().await.unwrap();
    }
}
