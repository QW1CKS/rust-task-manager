//! Profile-Guided Optimization (PGO) training workload (T348)
//!
//! Simulates typical usage patterns for PGO data collection.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use task_manager::core::filter::ProcessFilter;
use task_manager::core::process::{ProcessInfo, ProcessStore, ProcessStatus};
use task_manager::core::system::SystemMonitor;

/// Generate realistic process mix
fn generate_realistic_processes() -> Vec<ProcessInfo> {
    vec![
        ProcessInfo {
            pid: 4,
            name: "System".to_string(),
            cpu_usage: 2.5,
            memory_bytes: 256 * 1024,
            thread_count: 200,
            handle_count: 15000,
            priority: 8,
            status: ProcessStatus::Running,
        },
        ProcessInfo {
            pid: 720,
            name: "svchost.exe".to_string(),
            cpu_usage: 0.8,
            memory_bytes: 45 * 1024 * 1024,
            thread_count: 12,
            handle_count: 800,
            priority: 8,
            status: ProcessStatus::Running,
        },
        ProcessInfo {
            pid: 3456,
            name: "chrome.exe".to_string(),
            cpu_usage: 15.3,
            memory_bytes: 850 * 1024 * 1024,
            thread_count: 45,
            handle_count: 2500,
            priority: 8,
            status: ProcessStatus::Running,
        },
        ProcessInfo {
            pid: 7890,
            name: "code.exe".to_string(),
            cpu_usage: 8.2,
            memory_bytes: 450 * 1024 * 1024,
            thread_count: 28,
            handle_count: 1200,
            priority: 8,
            status: ProcessStatus::Running,
        },
        ProcessInfo {
            pid: 2468,
            name: "explorer.exe".to_string(),
            cpu_usage: 3.1,
            memory_bytes: 120 * 1024 * 1024,
            thread_count: 38,
            handle_count: 1800,
            priority: 8,
            status: ProcessStatus::Running,
        },
    ]
}

/// Simulate typical monitoring cycle
fn simulate_monitoring_cycle(iterations: usize) {
    let mut monitor = SystemMonitor::new();
    let mut store = ProcessStore::new();

    for _ in 0..iterations {
        let processes = generate_realistic_processes();
        store.update(&processes);
        
        // Simulate metric collection
        let _metrics = monitor.collect();
        
        black_box(&store);
        black_box(&monitor);
    }
}

/// Simulate filtering operations (most common user action)
fn simulate_filtering(iterations: usize) {
    let processes = generate_realistic_processes();
    let mut filter = ProcessFilter::new();

    for i in 0..iterations {
        // Vary filter patterns
        match i % 5 {
            0 => filter.set_name_filter("chrome"),
            1 => filter.set_name_filter("svchost"),
            2 => filter.set_name_filter(""),
            3 => filter.set_name_filter("explorer"),
            4 => filter.set_name_filter("code"),
            _ => {}
        }

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
            1 => processes.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes)),
            2 => processes.sort_by(|a, b| a.name.cmp(&b.name)),
            3 => processes.sort_by(|a, b| b.thread_count.cmp(&a.thread_count)),
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
    let mut store = ProcessStore::new();
    store.update(&processes);
    
    // System monitor initialization
    let _monitor = SystemMonitor::new();
    
    // UI initialization
    black_box("Initializing UI");
    
    black_box(store);
}

fn bench_typical_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("pgo_workload");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20);

    group.bench_function("monitoring_cycle_100", |b| {
        b.iter(|| simulate_monitoring_cycle(100));
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
            simulate_monitoring_cycle(50);
            simulate_filtering(100);
            simulate_sorting(50);
        });
    });

    group.finish();
}

fn bench_hot_paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("hot_paths");

    // Process enumeration hot path
    group.bench_function("process_enumeration", |b| {
        let mut store = ProcessStore::new();
        let processes = generate_realistic_processes();
        
        b.iter(|| {
            store.update(&processes);
            black_box(&store);
        });
    });

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
