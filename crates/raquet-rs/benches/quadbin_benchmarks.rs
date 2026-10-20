use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use arrow::array::{Float64Array, UInt8Array, UInt64Array, Int32Array};
use rand::Rng;
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

fn bench_quadbin_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Quadbin_Vectorized_Batch_Operations");
    
    // Scale tracking tests across varying array batch boundaries
    for batch_size in [1024, 4096, 16384].iter() {
        let (lon_arr, lat_arr, res_arr) = generate_random_coordinate_batch(*batch_size);
        
        // Generate pre-computed cell matrices for consumer pipeline tracking
        let cell_arr = QuadbinOrchestrator::quadbin_from_lonlat_batch(&lon_arr, &lat_arr, &res_arr).unwrap();
        let k_arr = Int32Array::from(vec![Some(1); *batch_size]);

        // Define throughput metrics in terms of elements/rows processed per iteration cycle
        group.throughput(Throughput::Elements(*batch_size as u64));

        // 1. Throughput for quadbin_from_lonlat_batch
        group.bench_with_input(
            BenchmarkId::new("quadbin_from_lonlat_batch", batch_size),
            batch_size,

            |b, _| {
                b.iter(|| {
                    QuadbinOrchestrator::quadbin_from_lonlat_batch(&lon_arr, &lat_arr, &res_arr).unwrap()
                });
            },
        );

        // 2. Throughput for quadbin_to_lonlat_batch
        group.bench_with_input(
            BenchmarkId::new("quadbin_to_lonlat_batch", batch_size),
            batch_size,

            |b, _| {
                b.iter(|| {
                    QuadbinOrchestrator::quadbin_to_lonlat_batch(&cell_arr).unwrap()
                });
            },
        );

        // 3. Throughput for quadbin_kring_batch
        group.bench_with_input(
            BenchmarkId::new("quadbin_kring_batch_r1", batch_size),
            batch_size,

            |b, _| {
                b.iter(|| {
                    QuadbinOrchestrator::quadbin_kring_batch(&cell_arr, &k_arr).unwrap()
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_quadbin_operations);
criterion_main!(benches);
