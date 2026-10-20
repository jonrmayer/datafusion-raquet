use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use arrow::array::{Array,Float64Array, UInt8Array, UInt64Array};
use rand::Rng;
use quadbin_rs::QuadBin;
use raquet_rs::quadbin::QuadbinOrchestrator;

fn generate_random_coordinate_batch(size: usize) -> (Float64Array, Float64Array, UInt8Array) {
    let mut rng = rand::thread_rng();
    let mut lons = Vec::with_capacity(size);
    let mut lats = Vec::with_capacity(size);
    let mut resolutions = Vec::with_capacity(size);

    for _ in 0..size {
        lons.push(Some(rng.gen_range(-180.0..180.0)));
        lats.push(Some(rng.gen_range(-85.0511..85.0511)));
        resolutions.push(Some(rng.gen_range(4..14)));
    }

    (
        Float64Array::from(lons),
        Float64Array::from(lats),
        UInt8Array::from(resolutions),
    )
}

/// Simulated row-by-row loop approach containing data-boxing overheads inside the execution frame.
fn row_by_row_simulation(lon_arr: &Float64Array, lat_arr: &Float64Array, res_arr: &UInt8Array) -> UInt64Array {
    let num_rows = lon_arr.len();
    let mut builder = UInt64Array::builder(num_rows);
    
    for i in 0..num_rows {
        if lon_arr.is_null(i) || lat_arr.is_null(i) || res_arr.is_null(i) {
            builder.append_null();
            continue;
        }
        // Mimicking DataFusion scalar unboxing layers by wrapping individual indices manually
        let lon = Some(lon_arr.value(i));
        let lat = Some(lat_arr.value(i));
        let res = Some(res_arr.value(i));
        
        if let (Some(ln), Some(lt), Some(rs)) = (lon, lat, res) {
            match QuadBin::from_lonlat(ln, lt, rs as i8) {
                Ok(quadbin) => builder.append_value(quadbin.cell()),
                Err(_) => builder.append_null(),
            }
        } else {
            builder.append_null();
        }
    }
    builder.finish()
}

fn bench_throughput_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("Quadbin_Row_vs_Vectorized_Throughput");
    
    // Scale evaluations across different load limits to trace architecture performance
    let batch_sizes = vec![1024, 8192];

    for batch_size in batch_sizes {
        let (lon_arr, lat_arr, res_arr) = generate_random_coordinate_batch(batch_size);

        // Instruct Criterion to compute row data velocity (Melem/s)
        group.throughput(Throughput::Elements(batch_size as u64));

        // Scenario A: Legacy row-by-row mapping loops
        group.bench_with_input(
            BenchmarkId::new("Row_by_Row_Simulated_UDF", batch_size),
            &batch_size,

            |b, _| {
                b.iter(|| row_by_row_simulation(&lon_arr, &lat_arr, &res_arr));
            },
        );

        // Scenario B: Pure, refactored Arrow-native orchestrator pathway
        group.bench_with_input(
            BenchmarkId::new("Vectorized_Arrow_Orchestrator", batch_size),
            &batch_size,

            |b, _| {
                b.iter(|| QuadbinOrchestrator::quadbin_from_lonlat_batch(&lon_arr, &lat_arr, &res_arr).unwrap());
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_throughput_comparison);
criterion_main!(benches);
