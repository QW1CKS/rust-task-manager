//! Startup performance benchmarks (SC-001 validation)

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_startup(_c: &mut Criterion) {
    // TODO: Implement startup benchmarks after Phase 2 (window creation)
}

criterion_group!(benches, bench_startup);
criterion_main!(benches);
