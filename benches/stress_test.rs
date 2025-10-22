//! Stress test benchmark simulating 1000 processes (T309)
//!
//! Tests system performance under high process count scenarios.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;
use task_manager::core::process::{ProcessInfo, ProcessStore};
use task_manager::core::system::SystemMonitor;

fn generate_mock_processes(count: usize) -> Vec<ProcessInfo> {
    (0..count)
        .map(|i| ProcessInfo {
            pid: i as u32,
            name: format!("process_{}.exe", i % 100), // 100 unique names
            cpu_usage: (i % 100) as f32,
            memory_bytes: (i * 1024 * 1024) as u64,
            thread_count: (i % 16 + 1) as u32,
            handle_count: (i % 500 + 100) as u32,
            priority: 8,
            status: task_manager::core::process::ProcessStatus::Running,
        })
        .collect()
}

fn bench_process_store_with_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("stress_test");
    group.measurement_time(Duration::from_secs(10));

    for process_count in [100, 500, 1000, 2000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("process_store_update", process_count),
            process_count,
            |b, &count| {
                let mut store = ProcessStore::new();
                let processes = generate_mock_processes(count);

                b.iter(|| {
                    store.update(&processes);
                    black_box(&store);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("process_enumeration", process_count),
            process_count,
            |b, &count| {
                let processes = generate_mock_processes(count);

                b.iter(|| {
                    let filtered: Vec<_> = processes
                        .iter()
                        .filter(|p| p.cpu_usage > 10.0)
                        .collect();
                    black_box(filtered);
                });
            },
        );
    }

    group.finish();
}

fn bench_sorting_large_process_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("sorting");
    
    for process_count in [1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("sort_by_cpu", process_count),
            process_count,
            |b, &count| {
                let mut processes = generate_mock_processes(count);

                b.iter(|| {
                    processes.sort_by(|a, b| {
                        b.cpu_usage
                            .partial_cmp(&a.cpu_usage)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    black_box(&processes);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("sort_by_memory", process_count),
            process_count,
            |b, &count| {
                let mut processes = generate_mock_processes(count);

                b.iter(|| {
                    processes.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
                    black_box(&processes);
                });
            },
        );
    }

    group.finish();
}

fn bench_filtering_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("filtering");
    let processes = generate_mock_processes(5000);

    group.bench_function("filter_by_name_substring", |b| {
        b.iter(|| {
            let filtered: Vec<_> = processes
                .iter()
                .filter(|p| p.name.contains("50"))
                .collect();
            black_box(filtered);
        });
    });

    group.bench_function("filter_by_cpu_threshold", |b| {
        b.iter(|| {
            let filtered: Vec<_> = processes
                .iter()
                .filter(|p| p.cpu_usage > 50.0)
                .collect();
            black_box(filtered);
        });
    });

    group.bench_function("filter_by_memory_threshold", |b| {
        b.iter(|| {
            let filtered: Vec<_> = processes
                .iter()
                .filter(|p| p.memory_bytes > 500 * 1024 * 1024)
                .collect();
            black_box(filtered);
        });
    });

    group.finish();
}

fn bench_memory_footprint(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory");
    group.sample_size(10);

    group.bench_function("allocate_1000_processes", |b| {
        b.iter(|| {
            let processes = generate_mock_processes(1000);
            black_box(processes);
        });
    });

    group.bench_function("allocate_5000_processes", |b| {
        b.iter(|| {
            let processes = generate_mock_processes(5000);
            black_box(processes);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_process_store_with_load,
    bench_sorting_large_process_list,
    bench_filtering_performance,
    bench_memory_footprint,
);
criterion_main!(benches);
