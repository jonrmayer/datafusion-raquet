use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use raquet_rs::{RaquetOrchestrator};
use quadbin_rs::QuadBin;

fn bench_vectorized_pipelines(c: &mut Criterion) {
    // 1. Arrange: Define a high workload footprint size matching deep database rows
    let sample_size = 10_000;
    let resolution = 7i8;

    // Synthetic floating-point coordinate inputs (cycling around standard global bounds)
    let lons: Vec<f64> = (0..sample_size)
        .map(|i| -180.0 + (i as f64 * 0.036))
        .collect();
    let lats: Vec<f64> = (0..sample_size)
        .map(|i| -90.0 + (i as f64 * 0.018))
        .collect();

    // Destination memory array buffers allocated upfront
    let mut out_cells = vec![0u64; sample_size];

    // Initialize a unified configuration group
    let mut group = c.benchmark_group("raquet_vector_pipelines");
    group.throughput(Throughput::Elements(sample_size as u64));

    // =====================================================================
    // PIPELINE 1: COORDINATES -> RAW 64-BIT INTEGER CELL COERCION
    // =====================================================================
    group.bench_function(
        BenchmarkId::new("batch_lonlat_to_cells", sample_size),

        |b| {
            b.iter(|| {
                // Execute the explicit unwrapped loop pipeline
                let _ = RaquetOrchestrator::batch_lonlat_to_cells(
                    &lons,
                    &lats,
                    resolution,
                    &mut out_cells,
                );
            })
        },
    );

    // =====================================================================
    // PIPELINE 2: RAW 64-BIT INTEGER CELLS -> COORDINATE FLOATS
    // =====================================================================
    
    // Seed our cell input array with valid, type-constructed bits
    let mut input_cells = vec![0u64; sample_size];
    let _ = RaquetOrchestrator::batch_lonlat_to_cells(
        &lons,
        &lats,
        resolution,
        &mut input_cells,
    );

    let mut dest_lons = vec![0.0; sample_size];
    let mut dest_lats = vec![0.0; sample_size];

    group.bench_function(
        BenchmarkId::new("batch_cells_to_lonlat", sample_size),

        |b| {
            b.iter(|| {
                // Execute the zero-copy unwrapped loop pipeline
                let _ = RaquetOrchestrator::batch_cells_to_lonlat(
                    &input_cells,
                    &mut dest_lons,
                    &mut dest_lats,
                );
            })
        },
    );

    group.finish();
}

criterion_group!(benches, bench_vectorized_pipelines);
criterion_main!(benches);
