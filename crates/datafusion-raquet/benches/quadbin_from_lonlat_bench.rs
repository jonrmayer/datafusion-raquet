use std::sync::Arc;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use arrow::array::{ArrayRef, Float64Array, Int64Array};
use arrow_schema::{DataType, Field};
use datafusion_expr::{ColumnarValue, ScalarFunctionArgs, ScalarUDFImpl};
use datafusion_raquet::udf::quadbin::quadbin_from_lonlat_udf::QuadbinFromLonLatUDF;

fn bench_quadbin_from_lonlat_udf_invoke(c: &mut Criterion) {
    let sample_size = 50_000;
    let target_resolution = 6i64;

    // Arrange: Synthetic data streams
    let lons: Vec<f64> = (0..sample_size).map(|i| -180.0 + (i as f64 * 0.007)).collect();
    let lats: Vec<f64> = (0..sample_size).map(|i| -90.0 + (i as f64 * 0.0035)).collect();
    let resolutions = vec![target_resolution; sample_size];

    let lon_array = Arc::new(Float64Array::from(lons)) as ArrayRef;
    let lat_array = Arc::new(Float64Array::from(lats)) as ArrayRef;
    let res_array = Arc::new(Int64Array::from(resolutions)) as ArrayRef;

    let lon_field = Arc::new(Field::new("lon", DataType::Float64, false));
    let lat_field = Arc::new(Field::new("lat", DataType::Float64, false));
    let res_field = Arc::new(Field::new("resolution", DataType::Int64, false));
    let return_field = Arc::new(Field::new("", DataType::UInt64, false));
    let config_opts = Arc::new(datafusion::config::ConfigOptions::default());

    let udf = QuadbinFromLonLatUDF::new();
    
    let mut group = c.benchmark_group("raquet_udf_performance");
    group.throughput(Throughput::Elements(sample_size as u64));

    group.bench_function(
        BenchmarkId::new("quadbin_from_lonlat_udf", sample_size),

        |b| {
            b.iter(|| {
                let args = ScalarFunctionArgs {
                    args: vec![
                        ColumnarValue::Array(Arc::clone(&lon_array)),
                        ColumnarValue::Array(Arc::clone(&lat_array)),
                        ColumnarValue::Array(Arc::clone(&res_array)),
                    ],
                    arg_fields: vec![
                        Arc::clone(&lon_field),
                        Arc::clone(&lat_field),
                        Arc::clone(&res_field),
                    ],
                    number_rows: sample_size,
                    return_field: Arc::clone(&return_field),
                    config_options: Arc::clone(&config_opts),
                };
                
                let _ = udf.invoke_with_args(args).unwrap();
            })
        },
    );

    group.finish();
}

criterion_group!(benches, bench_quadbin_from_lonlat_udf_invoke);
criterion_main!(benches);
