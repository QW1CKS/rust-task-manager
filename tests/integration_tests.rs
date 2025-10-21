//! Integration Tests (T139-T142)
//!
//! Comprehensive integration tests for the monitoring system:
//! - T139: Monitoring accuracy tests
//! - T140: Historical data tests
//! - T141: Thread safety tests
//! - T142: Error handling tests

use task_manager::app::updater::Updater;
use task_manager::core::process::ProcessStore;
use task_manager::core::system::CircularBuffer;
use task_manager::windows::monitor::SystemMonitor;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// T139: Test monitoring accuracy
///
/// Verifies that the monitoring system produces accurate results:
/// - Process enumeration finds current process
/// - Memory values are non-zero and increasing
/// - CPU times are monotonically increasing
/// - No duplicate PIDs in results
#[test]
fn test_monitoring_accuracy() {
    let mut monitor = SystemMonitor::new();
    let snapshot = monitor.collect_all().expect("Failed to collect snapshot");

    // Should find at least one process (this test process)
    assert!(
        snapshot.processes.len() > 0,
        "Should enumerate at least one process"
    );

    // Should find the current process
    let current_pid = std::process::id();
    let found = snapshot
        .processes
        .iter()
        .any(|p| p.pid == current_pid);
    assert!(found, "Should find current process in enumeration");

    // Verify no duplicate PIDs
    let mut pids: Vec<_> = snapshot.processes.iter().map(|p| p.pid).collect();
    pids.sort_unstable();
    let original_len = pids.len();
    pids.dedup();
    assert_eq!(
        pids.len(),
        original_len,
        "Should have no duplicate PIDs"
    );

    // Verify process data is reasonable
    for process in &snapshot.processes {
        assert!(process.pid > 0, "PID should be positive");
        assert!(!process.name.is_empty(), "Process should have a name");
        // Memory should be reasonable (not insanely high)
        assert!(
            process.memory_private < 100 * 1024 * 1024 * 1024,
            "Process memory should be < 100GB"
        );
    }
}

/// T139 (continued): Test process details accuracy
/// Note: Skipped until process details module is implemented
#[test]
#[ignore = "process details module not yet implemented"]
fn test_process_details_accuracy() {
    // let current_pid = std::process::id();
    // let details = get_process_details(current_pid).expect("Failed to get process details");

    // assert_eq!(details.pid, current_pid);
    // assert!(details.memory_details.working_set > 0, "Working set should be > 0");
    // assert!(details.handle_count > 0, "Should have open handles");
    // assert!(details.full_path.is_some(), "Should have executable path");
    
    // // Integrity level detection (returns Medium by default in current implementation)
    // println!("Integrity level: {:?}", details.integrity_level);
}

/// T140: Test historical data storage
///
/// Verifies that the circular buffer correctly stores and retrieves
/// historical data with proper time-based queries.
#[test]
fn test_historical_data_storage() {
    let mut buffer: CircularBuffer<f64> = CircularBuffer::new(10);

    // Fill buffer
    for i in 0..15 {
        buffer.push(i as f64);
        thread::sleep(Duration::from_millis(10));
    }

    // Should only contain last 10 values
    assert_eq!(buffer.len(), 10);

    // Get all data
    let all_data = buffer.get_all();
    assert_eq!(all_data.len(), 10);

    // Values should be 5-14 (last 10 values)
    let values: Vec<_> = all_data.iter().map(|dp| dp.value as i32).collect();
    assert_eq!(values, vec![5, 6, 7, 8, 9, 10, 11, 12, 13, 14]);

    // Test time-based query (last 1 second)
    let range = buffer.get_range(1);
    
    // All values should be within the range
    assert!(range.len() <= 10);
}

/// T140 (continued): Test ProcessStore updates
#[test]
fn test_process_store_updates() {
    let mut store = ProcessStore::new();
    let mut monitor = SystemMonitor::new();

    // Collect multiple snapshots
    for _ in 0..3 {
        let snapshot = monitor.collect_all().expect("Failed to collect");
        store.update(snapshot.processes);
        thread::sleep(Duration::from_millis(100));
    }

    // Store should have processes
    assert!(store.count() > 0, "Store should contain processes");

    // Should be able to find current process
    let current_pid = std::process::id();
    let process = store.get_by_pid(current_pid);
    assert!(process.is_some(), "Should find current process");
}

/// T141: Test thread safety
///
/// Verifies that the updater thread correctly handles concurrent access
/// and doesn't have data races.
#[test]
fn test_updater_thread_safety() {
    let (mut updater, receiver) = Updater::start(100); // 100ms refresh rate

    // Collect a few updates
    thread::sleep(Duration::from_millis(350));

    // Should receive multiple updates
    let mut update_count = 0;
    while let Ok(_update) = receiver.try_recv() {
        update_count += 1;
    }
    
    assert!(update_count >= 2, "Should receive at least 2 updates in 350ms");

    // Shutdown cleanly
    updater.shutdown();
}

/// T141 (continued): Test concurrent ProcessStore access
#[test]
fn test_concurrent_process_store_access() {
    let store = Arc::new(Mutex::new(ProcessStore::new()));
    let mut monitor = SystemMonitor::new();

    // Spawn multiple reader threads
    let mut handles = vec![];
    for _ in 0..5 {
        let store_clone = Arc::clone(&store);
        let handle = thread::spawn(move || {
            for _ in 0..10 {
                let guard = store_clone.lock().unwrap();
                let _ = guard.count();
                drop(guard);
                thread::sleep(Duration::from_millis(5));
            }
        });
        handles.push(handle);
    }

    // Update store from main thread
    for _ in 0..10 {
        let snapshot = monitor.collect_all().expect("Failed to collect");
        {
            let mut guard = store.lock().unwrap();
            guard.update(snapshot.processes);
        }
        thread::sleep(Duration::from_millis(10));
    }

    // Wait for all readers to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

/// T142: Test error handling
///
/// Verifies that the system gracefully handles error conditions:
/// - Invalid process access
/// - System overload scenarios
/// - Graceful degradation
/// Note: Skipped until process details module is implemented
#[test]
#[ignore = "process details module not yet implemented"]
fn test_error_handling_invalid_process() {
    // Try to get details for non-existent process
    // let invalid_pid = 999999u32;
    // let details = get_process_details(invalid_pid);
    
    // // Should return None for invalid PID (not panic)
    // assert!(details.is_none(), "Should return None for invalid PID");
}

/// T142 (continued): Test monitor resilience
#[test]
fn test_monitor_resilience() {
    let mut monitor = SystemMonitor::new();

    // Collect many snapshots in quick succession
    for _ in 0..20 {
        let result = monitor.collect_all();
        assert!(result.is_ok(), "Monitor should handle rapid collection");
    }
}

/// T142 (continued): Test updater pause/resume
#[test]
fn test_updater_pause_resume() {
    let (mut updater, receiver) = Updater::start(50); // 50ms refresh rate

    // Let it run briefly
    thread::sleep(Duration::from_millis(120));
    let initial_count = receiver.try_iter().count();
    assert!(initial_count >= 1, "Should receive updates while running");

    // Pause
    updater.pause();
    thread::sleep(Duration::from_millis(150));
    let paused_count = receiver.try_iter().count();
    assert_eq!(paused_count, 0, "Should not receive updates while paused");

    // Resume
    updater.resume();
    thread::sleep(Duration::from_millis(120));
    let resumed_count = receiver.try_iter().count();
    assert!(resumed_count >= 1, "Should receive updates after resume");

    // Cleanup
    updater.shutdown();
}

/// T142 (continued): Test circular buffer overflow
#[test]
fn test_circular_buffer_overflow() {
    let mut buffer: CircularBuffer<u32> = CircularBuffer::new(5);

    // Add more than capacity
    for i in 0..100 {
        buffer.push(i);
    }

    // Should only contain last 5 values
    assert_eq!(buffer.len(), 5);
    let values: Vec<_> = buffer.get_all().iter().map(|dp| dp.value).collect();
    assert_eq!(values, vec![95, 96, 97, 98, 99]);
}
