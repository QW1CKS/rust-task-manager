//! Monitoring cycle performance benchmarks
//!
//! Performance targets from constitution:
//! - Full monitoring cycle: <20ms
//! - Process enumeration: <5ms for 1000 processes
//! - Memory collection: <1ms

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use task_manager::windows::monitor::{memory, nt_query::ProcessEnumerator, SystemMonitor};

fn bench_process_enumeration(c: &mut Criterion) {
    c.bench_function("process_enumeration", |b| {
        let mut enumerator = ProcessEnumerator::new();
        b.iter(|| {
            let processes = enumerator.enumerate_processes().unwrap();
            black_box(processes);
        });
    });
}

fn bench_memory_collection(c: &mut Criterion) {
    c.bench_function("memory_collection", |b| {
        b.iter(|| {
            let metrics = memory::get_memory_metrics().unwrap();
            black_box(metrics);
        });
    });
}

fn bench_full_monitoring_cycle(c: &mut Criterion) {
    c.bench_function("full_monitoring_cycle", |b| {
        let mut monitor = SystemMonitor::new();
        b.iter(|| {
            let snapshot = monitor.collect_all().unwrap();
            black_box(snapshot);
        });
    });
}

criterion_group!(
    benches,
    bench_process_enumeration,
    bench_memory_collection,
    bench_full_monitoring_cycle
);
criterion_main!(benches);
