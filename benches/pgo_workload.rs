//! Profile-Guided Optimization (PGO) training workload (T348)
//!
//! Simulates typical usage patterns for PGO data collection.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use task_manager::core::filter::{ProcessFilter, ProcessInfo};

/// Generate realistic process mix
fn generate_realistic_processes() -> Vec<ProcessInfo> {
    vec![
        ProcessInfo {
            pid: 4,
            parent_pid: 0,
            name: "System".to_string(),
            cpu_usage: 2.5,
            memory_private: 256 * 1024,
            memory_working_set: 256 * 1024,
            io_read_bytes: 0,
            io_write_bytes: 0,
            handle_count: 15000,
        },
        ProcessInfo {
            pid: 720,
            parent_pid: 4,
            name: "svchost.exe".to_string(),
            cpu_usage: 0.8,
            memory_private: 45 * 1024 * 1024,
            memory_working_set: 45 * 1024 * 1024,
            io_read_bytes: 0,
            io_write_bytes: 0,
            handle_count: 800,
        },
        ProcessInfo {
            pid: 3456,
            parent_pid: 720,
            name: "chrome.exe".to_string(),
            cpu_usage: 15.3,
            memory_private: 850 * 1024 * 1024,
            memory_working_set: 850 * 1024 * 1024,
            io_read_bytes: 0,
            io_write_bytes: 0,
            handle_count: 2500,
        },
        ProcessInfo {
            pid: 7890,
            parent_pid: 720,
            name: "code.exe".to_string(),
            cpu_usage: 8.2,
            memory_private: 450 * 1024 * 1024,
            memory_working_set: 450 * 1024 * 1024,
            io_read_bytes: 0,
            io_write_bytes: 0,
            handle_count: 1200,
        },
        ProcessInfo {
            pid: 2468,
            parent_pid: 4,
            name: "explorer.exe".to_string(),
            cpu_usage: 3.1,
            memory_private: 120 * 1024 * 1024,
            memory_working_set: 120 * 1024 * 1024,
            io_read_bytes: 0,
            io_write_bytes: 0,
            handle_count: 1800,
        },
    ]
}

/// Simulate filtering operations (most common user action)
fn simulate_filtering(iterations: usize) {
    let processes = generate_realistic_processes();

    for i in 0..iterations {
        // Vary filter patterns
        let filter = match i % 5 {
            0 => ProcessFilter::new().with_name("chrome"),
            1 => ProcessFilter::new().with_name("svchost"),
            2 => ProcessFilter::new(),
            3 => ProcessFilter::new().with_name("explorer"),
            4 => ProcessFilter::new().with_name("code"),
            _ => ProcessFilter::new(),
        };

        let filtered: Vec<_> = processes.iter().filter(|p| filter.matches(p)).collect();
        black_box(filtered);
    }
}

/// Simulate sorting operations
fn simulate_sorting(iterations: usize) {
    let mut processes = generate_realistic_processes();

    for i in 0..iterations {
        match i % 4 {
            0 => processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap()),
            1 => processes.sort_by(|a, b| b.memory_private.cmp(&a.memory_private)),
            2 => processes.sort_by(|a, b| a.name.cmp(&b.name)),
            3 => processes.sort_by(|a, b| b.handle_count.cmp(&a.handle_count)),
            _ => {}
        }
        black_box(&processes);
    }
}

/// Simulate startup sequence
fn simulate_startup() {
    // Window creation
    black_box("Creating window");
    
    // Initial process enumeration
    let processes = generate_realistic_processes();
    
    // UI initialization
    black_box("Initializing UI");
    
    black_box(processes);
}

fn bench_typical_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("pgo_workload");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20);

    group.bench_function("filtering_1000", |b| {
        b.iter(|| simulate_filtering(1000));
    });

    group.bench_function("filtering_operations_500", |b| {
        b.iter(|| simulate_filtering(500));
    });

    group.bench_function("sorting_operations_200", |b| {
        b.iter(|| simulate_sorting(200));
    });

    group.bench_function("startup_sequence", |b| {
        b.iter(|| simulate_startup());
    });

    // Combined workload
    group.bench_function("combined_realistic_workload", |b| {
        b.iter(|| {
            simulate_startup();
            simulate_filtering(100);
            simulate_sorting(50);
        });
    });

    group.finish();
}

fn bench_hot_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_paths");

    // Filtering hot path
    group.bench_function("filter_matching", |b| {
        let processes = generate_realistic_processes();
        let filter = ProcessFilter::new();
        
        b.iter(|| {
            let filtered: Vec<_> = processes.iter().filter(|p| filter.matches(p)).collect();
            black_box(filtered);
        });
    });

    // Sorting hot path
    group.bench_function("sort_by_cpu", |b| {
        let mut processes = generate_realistic_processes();
        
        b.iter(|| {
            processes.sort_by(|a, b| {
                b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap()
            });
            black_box(&processes);
        });
    });

    group.finish();
}

fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    // Allocation pattern
    group.bench_function("allocate_process_list", |b| {
        b.iter(|| {
            let processes = generate_realistic_processes();
            black_box(processes);
        });
    });

    // Clone pattern
    group.bench_function("clone_process_list", |b| {
        let processes = generate_realistic_processes();
        b.iter(|| {
            let cloned = processes.clone();
            black_box(cloned);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_typical_workload,
    bench_hot_paths,
    bench_memory_patterns,
);
criterion_main!(benches);
