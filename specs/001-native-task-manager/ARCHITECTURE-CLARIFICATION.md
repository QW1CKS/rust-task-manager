# Architecture Clarification - Data Flow & Threading Model

**Generated**: 2025-10-21  
**Purpose**: Resolve architectural ambiguities identified in cross-artifact analysis  
**Related Issues**: A1 (data ownership), A3 (UI sync), A4 (thread safety), F5 (threading model)

---

## 1. Data Flow Architecture

### 1.1 Overall System Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Application Boundary                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌──────────────┐         ┌──────────────┐         ┌─────────────┐ │
│  │   Windows    │         │     Core     │         │     UI      │ │
│  │   Platform   │  ────>  │   Business   │  ────>  │  Rendering  │ │
│  │   (Monitor)  │         │    Logic     │         │   (D2D)     │ │
│  └──────────────┘         └──────────────┘         └─────────────┘ │
│        │                         │                         │         │
│        │ Raw Windows             │ Processed               │ Visual  │
│        │ API Data                │ Metrics                 │ Output  │
│        ▼                         ▼                         ▼         │
│  ┌──────────────┐         ┌──────────────┐         ┌─────────────┐ │
│  │ NT Query     │         │ ProcessStore │         │ Frame       │ │
│  │ PDH Counters │         │ (SoA Layout) │         │ Renderer    │ │
│  │ DXGI GPU     │         │ HistoryBuf   │         │ (60+ FPS)   │ │
│  └──────────────┘         └──────────────┘         └─────────────┘ │
│                                                                       │
└─────────────────────────────────────────────────────────────────────┘

Data Flow Direction: STRICTLY LEFT-TO-RIGHT
- Windows → Core: SystemMonitor produces ProcessSnapshot
- Core → UI: ProcessStore provides read-only views
- NO CIRCULAR DEPENDENCIES: UI never writes to Core, Core never calls Windows directly
```

### 1.2 Detailed Component Responsibilities

#### SystemMonitor (src/windows/monitor/)
**Role**: Collect raw system metrics from Windows APIs  
**Produces**: `ProcessSnapshot` containing raw process data  
**Consumes**: Nothing (no dependencies on Core or UI)

```rust
// src/windows/monitor/mod.rs

pub struct SystemMonitor {
    nt_query: NtQueryCollector,
    pdh: PdhCollector,
    dxgi: DxgiCollector,
}

pub struct ProcessSnapshot {
    pub timestamp: Instant,
    pub processes: Vec<ProcessInfo>,  // Owned data
    pub system_cpu: f32,
    pub system_memory: MemoryInfo,
    pub gpu_info: Option<GpuInfo>,
}

impl SystemMonitor {
    /// Collects all metrics in one shot, returns owned snapshot
    /// 
    /// # Performance
    /// Target: <50ms for 2048 processes
    /// - NtQuerySystemInformation: ~5ms
    /// - PDH collection: ~2ms  
    /// - DXGI query: ~1ms
    /// 
    /// # Ownership
    /// Caller takes ownership of returned ProcessSnapshot.
    /// SystemMonitor retains no references to collected data.
    pub fn collect_all(&mut self) -> Result<ProcessSnapshot> {
        // Collect from all sources
        // Return owned snapshot
    }
}
```

#### ProcessStore (src/core/process.rs)
**Role**: Organize process data for efficient queries and rendering  
**Consumes**: `ProcessSnapshot` from SystemMonitor  
**Produces**: Read-only views for UI rendering

```rust
// src/core/process.rs

pub struct ProcessStore {
    // Structure of Arrays for cache efficiency
    pids: Box<[u32; 2048]>,
    names: Box<[String; 2048]>,
    cpu_usage: Box<[f32; 2048]>,
    memory_working_set: Box<[u64; 2048]>,
    // ... other metric arrays
    
    count: usize,  // Actual number of valid processes
    last_update: Instant,
}

impl ProcessStore {
    /// Updates internal state from new snapshot
    /// 
    /// # Ownership
    /// Takes ownership of snapshot, extracts data into SoA arrays.
    /// Snapshot is dropped after extraction (no allocation retained).
    /// 
    /// # Performance
    /// Target: <5ms to update 2048 entries
    /// Zero allocations (reuses existing arrays)
    pub fn update(&mut self, snapshot: ProcessSnapshot) {
        // Extract snapshot.processes into SoA arrays
        // Sort by PID for binary search
        // Update count
        // Drop snapshot (Vec freed)
    }
    
    /// Returns iterator over processes (read-only view)
    /// 
    /// # Safety
    /// Returned references valid until next update() call.
    /// UI should not hold references across update boundaries.
    pub fn iter(&self) -> impl Iterator<Item = ProcessView<'_>> {
        (0..self.count).map(move |i| ProcessView {
            pid: self.pids[i],
            name: &self.names[i],
            cpu_usage: self.cpu_usage[i],
            // ... other fields
        })
    }
    
    /// Fast PID lookup via binary search
    /// 
    /// # Performance
    /// O(log n) = ~11 comparisons for 2048 processes
    pub fn get_by_pid(&self, pid: u32) -> Option<ProcessView<'_>> {
        // Binary search on sorted pids array
    }
}
```

#### UI Renderer (src/ui/d2d/renderer.rs)
**Role**: Render visual output from ProcessStore data  
**Consumes**: Read-only views from ProcessStore  
**Produces**: Pixels on screen via Direct2D

```rust
// src/ui/d2d/renderer.rs

pub struct Renderer {
    device_context: ID2D1DeviceContext,
    resources: ResourcePool,
}

impl Renderer {
    /// Renders one frame
    /// 
    /// # Performance
    /// Target: <16ms (60 FPS), stretch: <8ms (120 FPS)
    /// 
    /// # Data Access
    /// Reads from ProcessStore via read-only references.
    /// NEVER modifies ProcessStore or triggers updates.
    pub fn render(&mut self, store: &ProcessStore) -> Result<()> {
        unsafe { self.device_context.BeginDraw() };
        
        // Render process list
        for (i, process) in store.iter().enumerate() {
            self.draw_process_row(i, process);
        }
        
        // Render graphs, etc.
        
        unsafe { self.device_context.EndDraw(None, None) }?;
        Ok(())
    }
}
```

### 1.3 Data Ownership Rules

| Component | Owns Data? | Lifetime | Access Pattern |
|-----------|-----------|----------|----------------|
| **SystemMonitor** | ✅ Owns `ProcessSnapshot` until returned | Short (collect → return) | Produce & transfer |
| **ProcessStore** | ✅ Owns SoA arrays | Long (application lifetime) | Consume & store |
| **Renderer** | ❌ Borrows read-only | Short (frame duration) | Borrow & render |
| **ProcessSnapshot** | ✅ Owns `Vec<ProcessInfo>` | Short (collect → update) | Transfer container |

**Critical Rule**: ProcessStore::update() is the ONLY mutation point. After update(), all data is immutable until next update().

---

## 2. Threading Model

### 2.1 Thread Architecture

```
┌────────────────────────────────────────────────────────────────────────┐
│                           Process Space                                 │
├────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────────────────────────────┐   ┌─────────────────────────┐ │
│  │         UI Thread (Main)            │   │   Background Thread     │ │
│  │  (Thread Priority: NORMAL)          │   │  (BELOW_NORMAL)         │ │
│  ├─────────────────────────────────────┤   ├─────────────────────────┤ │
│  │                                     │   │                         │ │
│  │  • Win32 Message Loop               │   │  • Metric Collection    │ │
│  │    - GetMessage/DispatchMessage     │   │    - SystemMonitor      │ │
│  │    - WM_PAINT → Render              │   │    - Timed loop (1Hz)   │ │
│  │    - WM_KEYDOWN → Handle input      │   │                         │ │
│  │                                     │   │  • Send via mpsc        │ │
│  │  • ProcessStore (read-only)         │   │    - Bounded queue      │ │
│  │    - Render from                    │   │    - Drop if full       │ │
│  │    - Query for details              │   │                         │ │
│  │                                     │   │                         │ │
│  │  • ProcessStore::update()           │◄──┼──• ProcessSnapshot      │ │
│  │    - Receive from channel           │   │                         │ │
│  │    - Update on WM_TIMER             │   │                         │ │
│  │                                     │   │                         │ │
│  │  • User Actions                     │   │                         │ │
│  │    - Terminate process              │   │                         │ │
│  │    - Change priority                │   │                         │ │
│  │    - Elevation prompts              │   │                         │ │
│  │                                     │   │                         │ │
│  └─────────────────────────────────────┘   └─────────────────────────┘ │
│           │                                           │                  │
│           │ mpsc::Receiver<ProcessSnapshot>          │                  │
│           │◄──────────────────────────────────────────┘                  │
│           │ Bounded (capacity = 2)                                       │
│           │ Drop oldest if full                                          │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘

Message Flow:
1. Background thread: collect() every 1 second → send(snapshot)
2. Channel buffers up to 2 snapshots (drop oldest if full)
3. UI thread: WM_TIMER (500ms) → try_recv() → update() → InvalidateRect()
4. UI thread: WM_PAINT → render() from current state
```

### 2.2 Thread Coordination

#### Background Thread (Collector)

```rust
// src/app/updater.rs

pub struct BackgroundUpdater {
    monitor: SystemMonitor,
    sender: mpsc::SyncSender<ProcessSnapshot>,
    refresh_rate: Duration,
    running: Arc<AtomicBool>,
}

impl BackgroundUpdater {
    pub fn spawn(refresh_rate: Duration) -> (Self, mpsc::Receiver<ProcessSnapshot>) {
        let (tx, rx) = mpsc::sync_channel(2);  // Bounded: max 2 snapshots queued
        
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        
        thread::Builder::new()
            .name("metric-collector".to_string())
            .spawn(move || {
                // Set thread priority to BELOW_NORMAL to avoid starving UI
                unsafe {
                    SetThreadPriority(
                        GetCurrentThread(),
                        THREAD_PRIORITY_BELOW_NORMAL
                    );
                }
                
                let mut monitor = SystemMonitor::new();
                let mut next_collect = Instant::now();
                
                while running_clone.load(Ordering::Relaxed) {
                    // Precise timing: sleep until exact next collection time
                    let now = Instant::now();
                    if now < next_collect {
                        thread::sleep(next_collect - now);
                    }
                    
                    // Collect metrics
                    match monitor.collect_all() {
                        Ok(snapshot) => {
                            // Try to send, but DON'T block if channel full
                            // This prevents background thread from blocking on slow UI
                            match tx.try_send(snapshot) {
                                Ok(()) => {},  // Sent successfully
                                Err(TrySendError::Full(_)) => {
                                    // UI is behind, drop this snapshot
                                    // UI will get next one
                                    tracing::warn!("Dropped snapshot: UI processing too slow");
                                }
                                Err(TrySendError::Disconnected(_)) => {
                                    // UI thread exited, stop collecting
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Collection failed: {}", e);
                            // Continue collecting (transient error)
                        }
                    }
                    
                    // Schedule next collection
                    next_collect += refresh_rate;
                }
            })
            .expect("Failed to spawn background thread");
        
        (Self { /* ... */ }, rx)
    }
    
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
```

#### UI Thread (Consumer)

```rust
// src/app/state.rs

pub struct AppState {
    process_store: ProcessStore,
    snapshot_receiver: mpsc::Receiver<ProcessSnapshot>,
    last_update: Instant,
}

impl AppState {
    /// Called on WM_TIMER (every 500ms)
    /// 
    /// Checks for new snapshots from background thread.
    /// If available, updates ProcessStore and triggers repaint.
    pub fn poll_updates(&mut self, hwnd: HWND) -> bool {
        let mut updated = false;
        
        // Drain all pending snapshots (use most recent)
        while let Ok(snapshot) = self.snapshot_receiver.try_recv() {
            self.process_store.update(snapshot);
            self.last_update = Instant::now();
            updated = true;
        }
        
        if updated {
            // Trigger repaint
            unsafe {
                InvalidateRect(hwnd, None, false);
            }
        }
        
        updated
    }
}

// In window procedure
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM
) -> LRESULT {
    match msg {
        WM_TIMER => {
            if let Some(state) = get_window_state(hwnd) {
                state.poll_updates(hwnd);
            }
            LRESULT(0)
        }
        
        WM_PAINT => {
            if let Some(state) = get_window_state(hwnd) {
                // Render from current ProcessStore state
                // This is ALWAYS safe: ProcessStore is not being mutated during render
                renderer.render(&state.process_store).ok();
            }
            LRESULT(0)
        }
        
        // ... other messages
    }
}
```

### 2.3 Thread Safety Guarantees

| Operation | Thread | Lock Required? | Synchronization Mechanism |
|-----------|--------|----------------|---------------------------|
| **Collect metrics** | Background | ❌ No | SystemMonitor is thread-local |
| **Send snapshot** | Background | ❌ No | mpsc channel (lock-free internally) |
| **Receive snapshot** | UI | ❌ No | try_recv() is non-blocking |
| **Update ProcessStore** | UI | ❌ No | Only UI thread mutates |
| **Render from ProcessStore** | UI | ❌ No | Only UI thread reads (no concurrent mutation) |
| **Terminate process** | UI | ❌ No | Win32 APIs are thread-safe |
| **Elevation prompt** | UI | ❌ No | Must be on UI thread (Win32 requirement) |

**Key Insight**: NO LOCKS REQUIRED. UI thread has exclusive ownership of ProcessStore. Background thread never accesses it.

### 2.4 Privileged Operation Handling

Some operations (e.g., terminate system process) require elevation. These MUST occur on UI thread.

```rust
// src/windows/process/control.rs

pub enum ProcessOperation {
    Terminate { pid: u32, force: bool },
    SetPriority { pid: u32, priority: PriorityClass },
    Suspend { pid: u32 },
}

pub enum OperationResult {
    Success,
    RequiresElevation,
    AccessDenied,
    ProcessNotFound,
}

impl ProcessController {
    /// Attempts operation, returns result indicating if elevation needed
    /// 
    /// # Thread Safety
    /// MUST be called from UI thread (elevation prompts require message loop)
    pub fn execute(&self, op: ProcessOperation) -> OperationResult {
        match op {
            ProcessOperation::Terminate { pid, force } => {
                // Try to open process
                let handle = match OpenProcess(PROCESS_TERMINATE, false, pid) {
                    Ok(h) => h,
                    Err(e) if e.code() == ERROR_ACCESS_DENIED => {
                        return OperationResult::RequiresElevation;
                    }
                    Err(_) => return OperationResult::ProcessNotFound,
                };
                
                // Terminate
                if unsafe { TerminateProcess(handle, 1) }.is_ok() {
                    OperationResult::Success
                } else {
                    OperationResult::AccessDenied
                }
            }
            // ... other operations
        }
    }
}

// In UI event handler
fn on_terminate_process_clicked(hwnd: HWND, pid: u32) {
    let controller = ProcessController::new();
    
    match controller.execute(ProcessOperation::Terminate { pid, force: false }) {
        OperationResult::Success => {
            // Show success notification
        }
        OperationResult::RequiresElevation => {
            // Prompt user to restart as admin
            let result = MessageBoxW(
                hwnd,
                w!("Administrator privileges required. Restart as administrator?"),
                w!("Elevation Required"),
                MB_YESNO | MB_ICONQUESTION
            );
            
            if result == IDYES {
                // Restart application with RunAs verb
                ShellExecuteW(
                    hwnd,
                    w!("runas"),
                    get_current_exe_path(),
                    None,
                    None,
                    SW_SHOWNORMAL
                );
                
                // Exit current instance
                PostQuitMessage(0);
            }
        }
        _ => {
            // Show error dialog
        }
    }
}
```

---

## 3. Error Propagation Strategy

### 3.1 Error Handling Layers

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 4: User-Facing Errors (UI Dialogs)                  │
│  - "Administrator privileges required"                      │
│  - "Failed to terminate process: Access denied"             │
│  - Action buttons: Retry, Cancel, Run as Admin              │
└────────────────┬────────────────────────────────────────────┘
                 │ maps from
┌────────────────┴────────────────────────────────────────────┐
│  Layer 3: Application Errors (src/core/error.rs)           │
│  - Error::AccessDenied { process_name, pid }                │
│  - Error::ProcessNotFound { pid }                           │
│  - Error::CollectionFailed { source }                       │
└────────────────┬────────────────────────────────────────────┘
                 │ wraps
┌────────────────┴────────────────────────────────────────────┐
│  Layer 2: Platform Errors (src/windows/error.rs)           │
│  - WindowsError::NtStatus(status_code)                      │
│  - WindowsError::Win32Error(error_code)                     │
│  - WindowsError::HResult(hr)                                │
└────────────────┬────────────────────────────────────────────┘
                 │ wraps
┌────────────────┴────────────────────────────────────────────┐
│  Layer 1: Raw Windows Errors                                │
│  - NTSTATUS codes (0xC0000005 = ACCESS_VIOLATION)           │
│  - Win32 error codes (ERROR_ACCESS_DENIED = 5)              │
│  - HRESULT codes (E_FAIL = 0x80004005)                      │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 Error Types

```rust
// src/core/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaskManagerError {
    #[error("Access denied: {operation} requires administrator privileges")]
    AccessDenied { operation: String },
    
    #[error("Process not found: PID {pid}")]
    ProcessNotFound { pid: u32 },
    
    #[error("Failed to collect system metrics")]
    CollectionFailed {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Rendering failed: {reason}")]
    RenderError { reason: String },
    
    #[error("Windows API error: {0}")]
    WindowsError(#[from] WindowsError),
}

// src/windows/error.rs

#[derive(Error, Debug)]
pub enum WindowsError {
    #[error("NTSTATUS error: 0x{0:08X}")]
    NtStatus(i32),
    
    #[error("Win32 error: {0}")]
    Win32Error(#[from] windows::core::Error),
    
    #[error("HRESULT error: 0x{0:08X}")]
    HResult(i32),
}
```

### 3.3 Error Handling Patterns

#### Pattern 1: Collect → Log → Continue

```rust
// Background thread: Don't crash on transient errors
match monitor.collect_all() {
    Ok(snapshot) => {
        tx.send(snapshot).ok();
    }
    Err(e) => {
        // Log error but continue collecting
        tracing::error!("Collection failed: {}", e);
        // Next iteration will try again
    }
}
```

#### Pattern 2: Try → Fallback

```rust
// Try primary API, fallback to secondary
let cpu_usage = match pdh.get_cpu_usage() {
    Ok(usage) => usage,
    Err(e) => {
        tracing::warn!("PDH failed, using fallback: {}", e);
        // Fallback to NtQuerySystemInformation
        nt_query.get_cpu_usage()?
    }
};
```

#### Pattern 3: Fail → Inform User

```rust
// UI operation: Show error to user
match controller.terminate_process(pid) {
    Ok(()) => {
        show_notification("Process terminated successfully");
    }
    Err(TaskManagerError::AccessDenied { .. }) => {
        show_error_dialog(
            "Access Denied",
            "Administrator privileges required to terminate this process.",
            vec![
                ("Run as Admin", Action::Elevate),
                ("Cancel", Action::None),
            ]
        );
    }
    Err(e) => {
        show_error_dialog(
            "Operation Failed",
            &format!("Failed to terminate process: {}", e),
            vec![("OK", Action::None)]
        );
    }
}
```

---

## 4. Implementation Checklist

### 4.1 Data Flow Implementation

- [ ] **T133a**: Define `ProcessSnapshot` struct in `src/core/process.rs`
- [ ] **T133b**: Document ownership in `SystemMonitor::collect_all()` docstring
- [ ] **T133c**: Define `ProcessInfo` struct (64-byte aligned for cache efficiency)
- [ ] **T133d**: Implement `ProcessStore::update(ProcessSnapshot)` consuming snapshot
- [ ] **T133e**: Implement `ProcessStore::iter()` returning read-only iterator
- [ ] **T133f**: Add integration test: collect → update → render pipeline

### 4.2 Threading Model Implementation

- [ ] **T134**: Implement `BackgroundUpdater::spawn()` with thread priority setting
- [ ] **T135**: Use `mpsc::sync_channel(2)` for bounded queue
- [ ] **T136**: Implement precise timing with `Instant` and `thread::sleep`
- [ ] **T137**: Implement pause/resume via `AtomicBool` flag
- [ ] **T138**: Implement graceful shutdown, join background thread
- [ ] **T138a**: Implement privilege operation queue (if needed in future)

### 4.3 Error Propagation Implementation

- [ ] **T320-T347**: Implement error handling infrastructure (see Task Definitions document)
- [ ] Define `TaskManagerError` and `WindowsError` enums
- [ ] Implement error mapping: Windows codes → user-friendly messages
- [ ] Add error logging to rotating file
- [ ] Implement error dialogs with recovery actions

---

## 5. Architecture Validation

### 5.1 Validation Tests

```rust
// tests/integration/data_flow_test.rs

#[test]
fn test_data_flow_no_circular_dependency() {
    // Ensure SystemMonitor has no dependency on ProcessStore
    // (compile-time check: if it compiles, this passes)
}

#[test]
fn test_ownership_transfer() {
    let mut monitor = SystemMonitor::new();
    let snapshot = monitor.collect_all().unwrap();
    
    // Snapshot is moved here
    let mut store = ProcessStore::new();
    store.update(snapshot);
    
    // snapshot is now invalid (moved)
    // Compile error if we try to use snapshot here
}

#[test]
fn test_thread_safety() {
    let (updater, rx) = BackgroundUpdater::spawn(Duration::from_millis(100));
    let mut store = ProcessStore::new();
    
    // Collect 10 snapshots
    for _ in 0..10 {
        if let Ok(snapshot) = rx.recv_timeout(Duration::from_secs(2)) {
            store.update(snapshot);
        }
    }
    
    updater.shutdown();
    
    // Verify no data races (Miri validation)
}
```

### 5.2 Performance Validation

```rust
// benches/data_flow.rs

fn bench_snapshot_transfer(c: &mut Criterion) {
    let mut monitor = SystemMonitor::new();
    let mut store = ProcessStore::new();
    
    c.bench_function("snapshot_transfer", |b| {
        b.iter(|| {
            let snapshot = monitor.collect_all().unwrap();
            store.update(snapshot);
        });
    });
    
    // Target: <55ms total (50ms collect + 5ms update)
}
```

---

## 6. Design Decisions & Rationale

### 6.1 Why No Locks?

**Decision**: Use message passing (mpsc) instead of shared state (Arc<Mutex<T>>)

**Rationale**:
1. **Simpler reasoning**: No deadlock concerns, no lock contention
2. **Better performance**: No lock acquisition overhead (critical for 60+ FPS rendering)
3. **Cache efficiency**: UI thread has exclusive access to ProcessStore (no false sharing)
4. **Constitution compliance**: Aligns with zero-allocation hot paths (no Arc allocations per frame)

**Trade-off**: Background thread can be "behind" if UI is slow. ACCEPTABLE: UI will catch up, and dropping frames is better than janky rendering.

### 6.2 Why Bounded Channel (capacity 2)?

**Decision**: Use `sync_channel(2)` instead of unbounded channel

**Rationale**:
1. **Memory safety**: Prevents unbounded growth if UI thread is blocked
2. **Latency control**: UI shows recent data (not stale data from 10 seconds ago)
3. **Backpressure**: If UI can't keep up, drop snapshots instead of queueing infinitely

**Trade-off**: May drop snapshots if UI is very slow. ACCEPTABLE: 1Hz refresh rate means missing one snapshot is minor.

### 6.3 Why ProcessSnapshot Owns Vec?

**Decision**: SystemMonitor returns owned `Vec<ProcessInfo>`, not `&[ProcessInfo]`

**Rationale**:
1. **Clear ownership**: No lifetime confusion, no borrow checker fights
2. **Transfer efficiency**: Vec is just 3 pointers (24 bytes), cheap to move
3. **Decoupling**: SystemMonitor can be dropped/replaced without affecting ProcessStore

**Trade-off**: One allocation per collection cycle. ACCEPTABLE: Allocation happens in background thread, not hot path. Constitution allows allocations in non-hot-path code.

---

## 7. Future Enhancements

### 7.1 Multi-Producer Support (Future)

If we add multiple collectors (e.g., separate GPU thread), use `mpsc::Receiver` with multiple `Sender` clones:

```rust
let (tx, rx) = mpsc::sync_channel(2);

// CPU metrics thread
let tx_cpu = tx.clone();
thread::spawn(move || {
    loop {
        tx_cpu.send(CpuSnapshot { /* ... */ }).ok();
    }
});

// GPU metrics thread  
let tx_gpu = tx.clone();
thread::spawn(move || {
    loop {
        tx_gpu.send(GpuSnapshot { /* ... */ }).ok();
    }
});

// UI thread
while let Ok(snapshot) = rx.recv() {
    match snapshot {
        Snapshot::Cpu(cpu) => { /* update CPU data */ }
        Snapshot::Gpu(gpu) => { /* update GPU data */ }
    }
}
```

### 7.2 Lock-Free Data Structures (Future Optimization)

If profiling shows channel is bottleneck (unlikely), consider lock-free alternatives:
- `crossbeam::queue::ArrayQueue` (bounded, lock-free)
- `crossbeam::queue::SegQueue` (unbounded, lock-free)

**Note**: Only optimize if measured bottleneck. Constitution principle: "Measure first, optimize later."

---

## Document Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-10-21 | 1.0 | Initial architecture clarification | Analysis Tool |

---

**Status**: ✅ Complete and Ready for Implementation  
**Next Action**: Review with team, incorporate into plan.md Phase 3 checkpoint
