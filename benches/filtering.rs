//! Rendering and Filtering Benchmarks (T192, T199)

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use task_manager::core::filter::{ProcessFilter, ProcessSorter, SortColumn, SortDirection, ProcessInfo};

fn make_test_process(id: u32) -> ProcessInfo {
    ProcessInfo {
        pid: id,
        parent_pid: id / 2,
        name: format!("process_{}.exe", id % 20),
        cpu_usage: (id as f64 * 1.234) % 100.0,
        memory_private: (id as u64 * 1_000_000) % 5_000_000_000,
        memory_working_set: (id as u64 * 800_000) % 4_000_000_000,
        io_read_bytes: id as u64 * 10_000,
        io_write_bytes: id as u64 * 5_000,
        handle_count: (id * 100) % 5000,
    }
}

fn generate_processes(count: usize) -> Vec<ProcessInfo> {
    (0..count as u32).map(make_test_process).collect()
}

/// T199: Benchmark filtering performance
fn bench_filter_by_name(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_by_name");
    
    for size in [100, 500, 1000, 2000].iter() {
        let processes = generate_processes(*size);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let filter = ProcessFilter::new().with_name("process_5");
                let filtered = filter.apply(black_box(&processes));
                black_box(filtered);
            });
        });
    }
    
    group.finish();
}

/// T199: Benchmark CPU threshold filtering
fn bench_filter_by_cpu(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_by_cpu");
    
    for size in [100, 500, 1000, 2000].iter() {
        let processes = generate_processes(*size);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let filter = ProcessFilter::new().with_cpu_threshold(25.0);
                let filtered = filter.apply(black_box(&processes));
                black_box(filtered);
            });
        });
    }
    
    group.finish();
}

/// T199: Benchmark memory threshold filtering
fn bench_filter_by_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_by_memory");
    
    for size in [100, 500, 1000, 2000].iter() {
        let processes = generate_processes(*size);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let filter = ProcessFilter::new().with_memory_threshold(100_000_000);
                let filtered = filter.apply(black_box(&processes));
                black_box(filtered);
            });
        });
    }
    
    group.finish();
}

/// T199: Benchmark sorting performance
fn bench_sort_by_cpu(c: &mut Criterion) {
    let mut group = c.benchmark_group("sort_by_cpu");
    
    for size in [100, 500, 1000, 2000].iter() {
        let processes = generate_processes(*size);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut procs = black_box(processes.clone());
                let sorter = ProcessSorter::new(SortColumn::Cpu, SortDirection::Descending);
                sorter.sort(&mut procs);
                black_box(procs);
            });
        });
    }
    
    group.finish();
}

/// T199: Benchmark combined filter + sort
fn bench_filter_and_sort(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_and_sort");
    
    for size in [100, 500, 1000, 2000].iter() {
        let processes = generate_processes(*size);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let filter = ProcessFilter::new().with_cpu_threshold(10.0);
                let mut filtered = filter.apply(black_box(&processes));
                let sorter = ProcessSorter::new(SortColumn::Memory, SortDirection::Descending);
                sorter.sort_refs(&mut filtered);
                black_box(filtered);
            });
        });
    }
    
    group.finish();
}

/// T192: Benchmark table row rendering calculations
fn bench_table_virtualization(c: &mut Criterion) {
    use task_manager::ui::controls::table::ProcessTable;
    
    let mut group = c.benchmark_group("table_virtualization");
    
    // Simulate table with different process counts
    for size in [100, 500, 1000, 2000].iter() {
        let processes: Vec<_> = (0..*size)
            .map(|i| task_manager::ui::controls::table::ProcessInfo {
                pid: i as u32,
                parent_pid: 0,
                name: format!("process_{}.exe", i),
                cpu_usage: (i as f64 * 1.5) % 100.0,
                memory_private: i as u64 * 1_000_000,
                memory_working_set: i as u64 * 800_000,
                handle_count: i as u32 * 10,
            })
            .collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            let table = ProcessTable::new();
            
            b.iter(|| {
                // Simulate rendering 50 visible rows
                let viewport_height = 1200.0; // Enough for ~50 rows
                let (start, end, visible) = table.calculate_visible_rows(
                    black_box(&processes),
                    viewport_height
                );
                
                // Simulate formatting visible rows
                for process in &processes[start..end] {
                    black_box(table.format_cell_text(SortColumn::Name, process));
                    black_box(table.format_cell_text(SortColumn::Cpu, process));
                    black_box(table.format_cell_text(SortColumn::Memory, process));
                }
                
                black_box((start, end, visible));
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_filter_by_name,
    bench_filter_by_cpu,
    bench_filter_by_memory,
    bench_sort_by_cpu,
    bench_filter_and_sort,
    bench_table_virtualization
);

criterion_main!(benches);
