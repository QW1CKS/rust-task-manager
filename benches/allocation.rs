//! Allocator benchmark comparing mimalloc vs system allocator (T338-T341)
//!
//! Validates allocation performance claims for common patterns.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::alloc::{GlobalAlloc, Layout, System};
use std::time::Duration;

// Test patterns typical of task manager workload
fn bench_small_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_allocations");
    group.measurement_time(Duration::from_secs(5));

    // Simulate process name allocations (< 64 bytes)
    group.bench_function("vec_string_alloc_32_bytes", |b| {
        b.iter(|| {
            let mut strings = Vec::with_capacity(1000);
            for i in 0..1000 {
                strings.push(format!("process_{}.exe", i));
            }
            black_box(strings);
        });
    });

    // Simulate ProcessInfo struct allocations (~128 bytes)
    group.bench_function("vec_struct_alloc_128_bytes", |b| {
        b.iter(|| {
            let mut vec = Vec::with_capacity(1000);
            for i in 0..1000 {
                vec.push((i, i as f32, i as u64, format!("name_{}", i)));
            }
            black_box(vec);
        });
    });

    group.finish();
}

fn bench_medium_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("medium_allocations");

    // Simulate circular buffer allocations (3600 * 4 bytes = 14.4KB)
    group.bench_function("vec_alloc_15kb", |b| {
        b.iter(|| {
            let vec: Vec<f32> = vec![0.0; 3600];
            black_box(vec);
        });
    });

    // Simulate process list allocations
    group.bench_function("vec_alloc_process_list", |b| {
        b.iter(|| {
            let mut vec = Vec::with_capacity(500);
            for i in 0..500 {
                vec.push((
                    i as u32,
                    format!("process_{}", i),
                    i as f32,
                    i as u64 * 1024,
                ));
            }
            black_box(vec);
        });
    });

    group.finish();
}

fn bench_reallocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("reallocation");

    // Simulate growing vec pattern (process list updates)
    group.bench_function("vec_grow_pattern", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            for i in 0..1000 {
                vec.push(format!("process_{}", i));
            }
            black_box(vec);
        });
    });

    // Simulate vec with known capacity
    group.bench_function("vec_with_capacity", |b| {
        b.iter(|| {
            let mut vec = Vec::with_capacity(1000);
            for i in 0..1000 {
                vec.push(format!("process_{}", i));
            }
            black_box(vec);
        });
    });

    group.finish();
}

fn bench_deallocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("deallocation");

    // Bulk deallocation (clearing process list)
    group.bench_function("bulk_deallocate_1000", |b| {
        b.iter(|| {
            let vec: Vec<String> = (0..1000).map(|i| format!("process_{}", i)).collect();
            drop(vec);
        });
    });

    // Incremental deallocation
    group.bench_function("incremental_deallocate", |b| {
        b.iter(|| {
            let mut vec: Vec<String> = (0..1000).map(|i| format!("process_{}", i)).collect();
            while !vec.is_empty() {
                vec.pop();
            }
        });
    });

    group.finish();
}

fn bench_mixed_workload(c: &mut Criterion) {
    // Simulate realistic task manager allocation pattern
    c.bench_function("mixed_task_manager_workload", |b| {
        b.iter(|| {
            // Allocate process list
            let mut processes: Vec<(u32, String, f32)> = (0..500)
                .map(|i| (i, format!("proc_{}.exe", i % 50), i as f32))
                .collect();

            // Update some entries (realloc strings)
            for i in 0..100 {
                processes[i].1 = format!("updated_proc_{}.exe", i);
            }

            // Sort (no alloc, just moves)
            processes.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

            // Filter (allocates new vec)
            let filtered: Vec<_> = processes.iter().filter(|p| p.2 > 250.0).collect();

            black_box(&processes);
            black_box(filtered);
        });
    });
}

fn bench_arena_pattern(c: &mut Criterion) {
    use std::cell::RefCell;

    thread_local! {
        static ARENA: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(1024 * 1024));
    }

    c.bench_function("arena_reuse_pattern", |b| {
        b.iter(|| {
            ARENA.with(|arena| {
                let mut arena = arena.borrow_mut();
                arena.clear();

                // Simulate temporary allocations for rendering
                for i in 0..1000 {
                    let bytes = format!("temp_data_{}", i).into_bytes();
                    arena.extend_from_slice(&bytes);
                }

                black_box(arena.len());
            });
        });
    });
}

// Raw allocation benchmarks for comparison
fn bench_raw_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("raw_allocations");

    group.bench_function("system_alloc_1kb", |b| {
        b.iter(|| {
            let layout = Layout::from_size_align(1024, 8).unwrap();
            unsafe {
                let ptr = System.alloc(layout);
                if !ptr.is_null() {
                    System.dealloc(ptr, layout);
                }
            }
        });
    });

    group.bench_function("system_alloc_16kb", |b| {
        b.iter(|| {
            let layout = Layout::from_size_align(16384, 8).unwrap();
            unsafe {
                let ptr = System.alloc(layout);
                if !ptr.is_null() {
                    System.dealloc(ptr, layout);
                }
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_small_allocations,
    bench_medium_allocations,
    bench_reallocation_patterns,
    bench_deallocation_patterns,
    bench_mixed_workload,
    bench_arena_pattern,
    bench_raw_allocations,
);
criterion_main!(benches);
