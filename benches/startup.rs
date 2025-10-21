//! Startup performance benchmarks (T147a-i)
//!
//! Measures cold start and initialization performance:
//! - Cold start time (process creation to first frame)
//! - ProcessStore initialization
//! - Window creation and Direct2D setup
//! - First data collection
//! - Memory footprint during startup

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use task_manager::core::process::ProcessStore;
use task_manager::core::system::CircularBuffer;
use task_manager::windows::monitor::SystemMonitor;
use task_manager::windows::monitor::pdh::SystemMetricsCollector;
use task_manager::windows::monitor::dxgi::GpuCollector;
use task_manager::windows::process::details::get_process_details;

/// T147a: Benchmark ProcessStore initialization
///
/// Target: <1ms for empty store allocation
fn bench_process_store_init(c: &mut Criterion) {
    c.bench_function("process_store_init", |b| {
        b.iter(|| {
            black_box(ProcessStore::new())
        });
    });
}

/// T147b: Benchmark CircularBuffer initialization
///
/// Target: <100μs for 3600-sample buffer
fn bench_circular_buffer_init(c: &mut Criterion) {
    c.bench_function("circular_buffer_init_3600", |b| {
        b.iter(|| {
            black_box(CircularBuffer::<f64>::new(3600))
        });
    });
}

/// T147c: Benchmark SystemMonitor initialization
///
/// Target: <5ms (includes 1MB buffer allocation)
fn bench_system_monitor_init(c: &mut Criterion) {
    c.bench_function("system_monitor_init", |b| {
        b.iter(|| {
            black_box(SystemMonitor::new())
        });
    });
}

/// T147d: Benchmark first process enumeration
///
/// Target: <10ms for first call (cold cache)
fn bench_first_process_enum(c: &mut Criterion) {
    c.bench_function("first_process_enumeration", |b| {
        b.iter(|| {
            let mut monitor = SystemMonitor::new();
            black_box(monitor.collect_all())
        });
    });
}

/// T147e: Benchmark ProcessStore first update
///
/// Target: <5ms for initial population
fn bench_process_store_first_update(c: &mut Criterion) {
    c.bench_function("process_store_first_update", |b| {
        b.iter(|| {
            let mut store = ProcessStore::new();
            let mut monitor = SystemMonitor::new();
            if let Ok(snapshot) = monitor.collect_all() {
                store.update(snapshot.processes);
            }
            black_box(store)
        });
    });
}

/// T147f: Benchmark process details collection
///
/// Target: <1ms for single process
fn bench_process_details(c: &mut Criterion) {
    let pid = std::process::id();
    
    c.bench_function("process_details_collection", |b| {
        b.iter(|| {
            black_box(get_process_details(pid))
        });
    });
}

/// T147g: Benchmark PDH collector initialization
///
/// Target: <50ms (PDH counter initialization can be slow)
fn bench_pdh_init(c: &mut Criterion) {
    c.bench_function("pdh_collector_init", |b| {
        b.iter(|| {
            // May fail on non-English systems, that's okay
            let _ = black_box(SystemMetricsCollector::new());
        });
    });
}

/// T147h: Benchmark GPU collector initialization
///
/// Target: <20ms (DXGI initialization)
fn bench_gpu_init(c: &mut Criterion) {
    c.bench_function("gpu_collector_init", |b| {
        b.iter(|| {
            // May fail on systems without GPU/DXGI support
            let _ = black_box(GpuCollector::new());
        });
    });
}

/// T147i: Benchmark full initialization pipeline
///
/// Target: <100ms for complete cold start (excluding window creation)
fn bench_full_init_pipeline(c: &mut Criterion) {
    c.bench_function("full_initialization_pipeline", |b| {
        b.iter(|| {
            // ProcessStore
            let mut store = ProcessStore::new();
            
            // Circular buffers (1 hour history)
            let _cpu_history = CircularBuffer::<f64>::new(3600);
            let _memory_history = CircularBuffer::<u64>::new(3600);
            
            // SystemMonitor
            let mut monitor = SystemMonitor::new();
            
            // First data collection
            if let Ok(snapshot) = monitor.collect_all() {
                store.update(snapshot.processes);
            }
            
            black_box(store)
        });
    });
}

criterion_group!(
    benches,
    bench_process_store_init,
    bench_circular_buffer_init,
    bench_system_monitor_init,
    bench_first_process_enum,
    bench_process_store_first_update,
    bench_process_details,
    bench_pdh_init,
    bench_gpu_init,
    bench_full_init_pipeline,
);
criterion_main!(benches);
