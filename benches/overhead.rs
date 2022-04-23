use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tracing::{info, instrument};
use tracing_subscriber::prelude::*;

#[instrument]
fn fibonacci(n: u64) -> u64 {
    if n < 2 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fn instrument_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("instrument");
    group.throughput(criterion::Throughput::Elements(3));

    group.bench_function("locations_and_args", |b| {
        let (layer, _guard) = tracing_chrome::ChromeLayerBuilder::new()
            .include_args(true)
            .include_locations(true)
            .writer(std::io::sink())
            .build();
        let _subscriber = tracing_subscriber::registry().with(layer).set_default();
        b.iter(|| black_box(fibonacci(3)));
    });
    group.bench_function("locations", |b| {
        let (layer, _guard) = tracing_chrome::ChromeLayerBuilder::new()
            .include_args(false)
            .include_locations(true)
            .writer(std::io::sink())
            .build();
        let _subscriber = tracing_subscriber::registry().with(layer).set_default();
        b.iter(|| black_box(fibonacci(3)));
    });
    group.bench_function("minimal", |b| {
        let (layer, _guard) = tracing_chrome::ChromeLayerBuilder::new()
            .include_args(false)
            .include_locations(false)
            .writer(std::io::sink())
            .build();
        let _subscriber = tracing_subscriber::registry().with(layer).set_default();
        b.iter(|| black_box(fibonacci(3)));
    });
}

fn event_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("event");
    group.throughput(criterion::Throughput::Elements(1));

    group.bench_function("locations_and_args", |b| {
        let (layer, _guard) = tracing_chrome::ChromeLayerBuilder::new()
            .include_args(true)
            .include_locations(true)
            .writer(std::io::sink())
            .build();
        let _subscriber = tracing_subscriber::registry().with(layer).set_default();
        b.iter(|| {
            info!(arg = 42, "Something Happen");
        });
    });
    group.bench_function("locations", |b| {
        let (layer, _guard) = tracing_chrome::ChromeLayerBuilder::new()
            .include_args(false)
            .include_locations(true)
            .writer(std::io::sink())
            .build();
        let _subscriber = tracing_subscriber::registry().with(layer).set_default();
        b.iter(|| {
            info!(arg = 42, "Something Happen");
        });
    });
    group.bench_function("minimal", |b| {
        let (layer, _guard) = tracing_chrome::ChromeLayerBuilder::new()
            .include_args(false)
            .include_locations(false)
            .writer(std::io::sink())
            .build();
        let _subscriber = tracing_subscriber::registry().with(layer).set_default();
        b.iter(|| {
            info!(arg = 42, "Something Happen");
        });
    });
}

criterion_group!(benches, instrument_benchmark, event_benchmark);
criterion_main!(benches);
