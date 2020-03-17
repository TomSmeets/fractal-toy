use criterion::{black_box, criterion_group, criterion_main, Criterion};

use serial::fractal::tile::{TileContent, TilePos};

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut g = c.benchmark_group("fractal");
    g.sample_size(20);
    g.bench_function("TileContent at root", |b| {
        b.iter(|| TileContent::new(TilePos::root()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
