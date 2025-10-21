# CRITICAL Issue Resolution - Immediate Edits Required

**Generated**: 2025-10-21  
**Purpose**: Provide exact file edits to resolve 5 CRITICAL issues before Phase 3  
**Status**: READY TO APPLY

---

## Issue F1: Fixed Array Size Insufficient (CRITICAL)

**Problem**: Constitution requires support for 2048 processes, but T078 hardcodes 1024-element arrays.

**Impact**: Application will crash or truncate data on enterprise servers with high process counts.

### Edit 1: Update tasks.md T078

**File**: `specs/001-native-task-manager/tasks.md`  
**Location**: Line ~174 (search for "T078")

**OLD TEXT**:
```markdown
- [ ] T078 [CRITICAL] [PERF] [US1] [US2] Define fixed-size arrays: pids: Box<[u32; 1024]>, names: Box<[String; 1024]> in src/core/process.rs
```

**NEW TEXT**:
```markdown
- [ ] T078 [CRITICAL] [PERF] [US1] [US2] Define fixed-size arrays with constitutional capacity: pids: Box<[u32; 2048]>, names: Box<[String; 2048]>, process_count: usize in src/core/process.rs
- [ ] T078a [CRITICAL] Add compile-time capacity assertion: const_assert!(MAX_PROCESSES == 2048) using static_assertions crate to prevent accidental reduction
- [ ] T078b [PERF] Document memory layout in code comments: 2048 processes × ~200 bytes/process = ~410KB for SoA storage (within <15MB idle budget)
```

### Edit 2: Update all related tasks (T079-T085)

**File**: `specs/001-native-task-manager/tasks.md`  
**Location**: Lines ~175-185

**Replace each occurrence of `1024` with `2048` in tasks T079 through T085**:

**OLD TEXT** (example from T079):
```markdown
- [ ] T079 [PERF] [US1] [US2] Add CPU metrics arrays: cpu_usage: Box<[f32; 1024]>, cpu_time_user: Box<[u64; 1024]> in src/core/process.rs
```

**NEW TEXT**:
```markdown
- [ ] T079 [PERF] [US1] [US2] Add CPU metrics arrays: cpu_usage: Box<[f32; 2048]>, cpu_time_user: Box<[u64; 2048]> in src/core/process.rs
```

**Repeat for T080, T081, T082** (change all `1024` to `2048`)

### Edit 3: Add Cargo.toml dependency

**File**: `plan.md` (will flow to Cargo.toml during implementation)  
**Location**: Section "Primary Dependencies" (~line 30)

**ADD AFTER** `bumpalo` line:
```toml
static_assertions = "1.1"   # Compile-time assertions for capacity validation
```

---

## Issue N3: Mica/Acrylic Implementation Missing (CRITICAL)

**Problem**: FR-043 requires Mica/Acrylic materials, but NO tasks implement Windows.UI.Composition integration.

**Impact**: Missing key visual differentiator, spec non-compliance.

### Edit 4: Insert Mica/Acrylic task group in tasks.md

**File**: `specs/001-native-task-manager/tasks.md`  
**Location**: After T045 (line ~111, search for "T045 [PERF] Implement Present1")

**INSERT AFTER T045**:
```markdown

### Windows 11 Fluent Design Materials

- [ ] T045a [WINRT] [US1] Implement src/ui/d2d/composition.rs with Windows.UI.Composition interop via CreateDispatcherQueueController
- [ ] T045b [WINRT] Create Compositor instance and CompositionTarget for HWND using Compositor::CreateTargetForDesktop in src/ui/d2d/composition.rs
- [ ] T045c [WINRT] [US1] Implement Mica backdrop: Create DesktopAcrylicBackdrop (Windows 11 22H2+) with MicaBackdrop fallback (Windows 11 21H2) in src/ui/d2d/composition.rs
- [ ] T045d [WINRT] Apply Acrylic to background panels using CompositionBrush with blur effect (BackdropBrush + EffectFactory) in src/ui/d2d/composition.rs
- [ ] T045e [WIN32] Implement OS version detection: RtlGetVersion() wrapper returning bool for Windows 11+, cache result in src/windows/version.rs
- [ ] T045f [US1] Implement automatic degradation: If Windows 10 (version.is_windows_11() == false), skip composition setup entirely and use solid color fill (no Mica/Acrylic), no user notification per FR-043
- [ ] T045g [P] Add debug toggle to disable composition for performance testing: Feature flag "fluent-ui" (enabled by default), allows clean perf baseline measurement
- [ ] T045h [P] Handle composition failures gracefully: If CreateDispatcherQueueController fails, fall back to solid colors and log warning (don't crash)

```

### Edit 5: Update Phase 2 checkpoint

**File**: `specs/001-native-task-manager/tasks.md`  
**Location**: Line ~147 (search for "Checkpoint Phase 2")

**OLD TEXT**:
```markdown
**Checkpoint Phase 2**: Window opens in <100ms, renders solid color background at 60+ FPS, responds to mouse/keyboard input, handles DPI changes
```

**NEW TEXT**:
```markdown
**Checkpoint Phase 2**: Window opens in <100ms, renders background (Mica on Windows 11, solid color on Windows 10) at 60+ FPS, responds to mouse/keyboard input, handles DPI changes without restart
```

---

## Issue F2: No Startup Time Measurement (CRITICAL)

**Problem**: SC-001 requires <500ms cold start but NO task measures or validates this.

**Impact**: Cannot verify primary performance claim; may ship slow startup without knowing.

### Edit 6: Insert startup benchmark tasks

**File**: `specs/001-native-task-manager/tasks.md`  
**Location**: After T147 (line ~273, search for "T147 [PERF] Add criterion regression detection")

**INSERT AFTER T147**:
```markdown

### Startup Performance Validation (SC-001)

- [ ] T147a [CRITICAL] [PERF] Create benches/startup.rs measuring cold start end-to-end: spawn process → measure time until first UI frame rendered via named pipe signal
- [ ] T147b [PERF] Benchmark Win32 window creation separately: Measure CreateWindowExW → RegisterClassExW → ShowWindow cycle (target <50ms)
- [ ] T147c [PERF] Benchmark Direct2D initialization separately: Measure D2D1CreateFactory → CreateRenderTarget → first BeginDraw (target <80ms, includes D3D11 device creation)
- [ ] T147d [PERF] Benchmark initial data collection: First NtQuerySystemInformation + PDH setup (target <100ms for first snapshot)
- [ ] T147e [PERF] Benchmark first frame render: Measure first BeginDraw → EndDraw → Present with minimal content (target <16ms for empty frame)
- [ ] T147f [PERF] Add startup timeline instrumentation: Emit named pipe events for "WinMain_entry", "window_created", "d2d_ready", "data_loaded", "first_paint_complete"
- [ ] T147g [CRITICAL] Validate sum of components <500ms per SC-001: Assert total startup time <500ms on mid-range reference system, fail CI if exceeded by >10% (>550ms)
- [ ] T147h [P] Add startup flamegraph generation for optimization: Use cargo flamegraph --bench startup, upload to CI artifacts for regression analysis
- [ ] T147i [P] Benchmark warm start (from cache): Measure startup with file system cache primed, target <200ms per plan.md budget

```

### Edit 7: Update plan.md startup budget

**File**: `specs/001-native-task-manager/plan.md`  
**Location**: ~Line 40 (search for "Startup Budget")

**OLD TEXT**:
```markdown
**Startup Budget**: <200ms Win32/D2D initialization, <100ms data collection, <200ms UI render
```

**NEW TEXT**:
```markdown
**Startup Budget**: <200ms Win32/D2D initialization (50ms window + 80ms D2D + 70ms margin), <100ms data collection, <200ms UI render, total <500ms per SC-001

**Validation**: Benchmarks in benches/startup.rs measure each component separately and validate sum. CI fails if any component exceeds budget by >10%.
```

---

## Issue G1: Per-Monitor DPI v2 Not Fully Implemented (CRITICAL)

**Problem**: FR-047 requires complete per-monitor DPI v2 support, but tasks only handle basic WM_DPICHANGED message. Missing non-client area scaling, child window propagation, resource scaling.

**Impact**: Blurry UI on mixed-DPI setups (common in enterprise: laptop + external 4K monitor).

### Edit 8: Insert DPI v2 complete implementation tasks

**File**: `specs/001-native-task-manager/tasks.md`  
**Location**: After T050 (line ~119, search for "T050 [P] Implement SetProcessDpiAwarenessContext")

**INSERT AFTER T050**:
```markdown

### Per-Monitor DPI v2 Complete Implementation (FR-047)

- [ ] T050a [WIN32] Set DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2 in application manifest: Generate manifest in build.rs with <dpiAwareness>PerMonitorV2</dpiAwareness>
- [ ] T050b [WIN32] Implement non-client area DPI scaling: Override WM_NCCALCSIZE to adjust title bar and window border thickness based on GetDpiForWindow
- [ ] T050c [CRITICAL] Implement DPI virtualization for child controls: When WM_DPICHANGED received, iterate all Control trait implementers and call set_dpi(new_dpi) method
- [ ] T050d [PERF] Scale Direct2D resources per-monitor: On DPI change, recreate all brushes, fonts, and geometries at new DPI in d2d/resources.rs (call recreate_for_dpi(dpi))
- [ ] T050e [WIN32] Implement icon resource scaling: Load appropriate icon size from resources (16x16 @ 96 DPI → 24x24 @ 144 DPI → 32x32 @ 192 DPI) using LoadIconWithScaleDown
- [ ] T050f [WIN32] Scale window non-client metrics: Use GetSystemMetricsForDpi for SM_CYCAPTION, SM_CXSIZEFRAME to ensure title bar and borders scale correctly
- [ ] T050g [CRITICAL] Add integration test for DPI changes: Simulate WM_DPICHANGED with different DPI values (96, 120, 144, 192), verify no blurry rendering or layout issues
- [ ] T050h [P] Add DPI change animation (polish): Smooth transition over 200ms when window moves between monitors using composition animation (if time permits)

```

### Edit 9: Update Phase 2 DPI checkpoint

**File**: `specs/001-native-task-manager/tasks.md`  
**Location**: Line ~147 (Checkpoint Phase 2, same location as Edit 5)

**MODIFY** the checkpoint text to:
```markdown
**Checkpoint Phase 2**: Window opens in <100ms, renders background (Mica on Windows 11, solid color on Windows 10) at 60+ FPS, responds to mouse/keyboard input, handles per-monitor DPI v2 changes (including non-client area scaling) without restart or blur
```

---

## Issue A1: Data Ownership Circular Dependency Risk (CRITICAL)

**Problem**: SystemMonitor and ProcessStore have unclear ownership. Risk of circular dependencies or lifetime issues.

**Impact**: Cannot implement Phase 3 safely without clear data flow specification.

### Edit 10: Insert data ownership specification tasks

**File**: `specs/001-native-task-manager/tasks.md`  
**Location**: After T133 (line ~250, search for "T133 [P] [US1] Add configurable refresh rate")

**INSERT AFTER T133**:
```markdown

### Data Flow and Ownership Specification (Critical for Phase 3)

- [ ] T133a [CRITICAL] Define ProcessSnapshot struct in src/core/process.rs: Contains timestamp: Instant, processes: Vec<ProcessInfo>, system_cpu: f32, system_memory: MemoryInfo
- [ ] T133b Document ownership model in SystemMonitor: collect_all() returns owned ProcessSnapshot (caller takes ownership), SystemMonitor retains no references to collected data
- [ ] T133c [PERF] Implement ProcessInfo struct in src/core/process.rs: Contains only essential fields (pid: u32, name: String, cpu_usage: f32, memory_working_set: u64), sized ~64 bytes for cache efficiency
- [ ] T133d Add transformation layer: SystemMonitor::collect_all() returns Result<ProcessSnapshot>, ProcessStore::update(snapshot) consumes snapshot and updates SoA arrays
- [ ] T133e [PERF] Optimize transfer: ProcessStore::update() takes ownership of Vec, extracts data into SoA arrays, drops Vec (no reallocation during transfer)
- [ ] T133f Document error handling: If collect_all() fails, ProcessStore retains previous state, UI shows last-known-good data with staleness indicator ("Data from 2 seconds ago")
- [ ] T133g [CRITICAL] Add integration test: Test data flow SystemMonitor → ProcessStore → Renderer, verify no circular dependencies, no dangling references, no data races (validate with Miri)

```

### Edit 11: Update plan.md with data flow diagram

**File**: `specs/001-native-task-manager/plan.md`  
**Location**: After "Project Structure" section (~line 180)

**INSERT NEW SECTION**:
```markdown

## Data Flow Architecture

### Component Ownership

```
┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐
│ SystemMonitor   │ ────>   │ ProcessStore    │ ────>   │ Renderer        │
│ (Background)    │produces │ (UI Thread)     │provides │ (UI Thread)     │
└─────────────────┘         └─────────────────┘         └─────────────────┘
        │                            │                            │
    Owns Vec                     Owns SoA                    Borrows &
    ProcessInfo                   Arrays                     (read-only)
```

**Ownership Rules**:
1. SystemMonitor produces owned `ProcessSnapshot` containing `Vec<ProcessInfo>`
2. ProcessStore consumes `ProcessSnapshot`, transfers ownership of `Vec`
3. ProcessStore extracts data into SoA arrays, drops `Vec`
4. Renderer borrows read-only references from ProcessStore (never mutates)
5. NO circular dependencies: Windows → Core → UI (strictly unidirectional)

**See**: `ARCHITECTURE-CLARIFICATION.md` for complete threading and data flow specification.

```

---

## Issue N1: Module Naming Inconsistency (Minor but Easy Fix)

**Problem**: Constitution defines `windows-sys/` module but plan.md uses `windows/` directory.

**Impact**: Low - naming confusion, but doesn't block implementation.

### Edit 12: Clarify naming in constitution

**File**: `.specify/memory/constitution.md`  
**Location**: ~Line 228 (search for "windows-sys/")

**OLD TEXT**:
```markdown
rust-task-manager/
├── core/              # Core system monitoring (no UI dependencies)
├── windows-sys/       # Windows API wrappers and utilities
├── ui/                # UI framework and rendering
```

**NEW TEXT**:
```markdown
rust-task-manager/
├── core/              # Core system monitoring (no UI dependencies)
├── windows/           # Windows API wrappers and utilities (uses windows-rs crate)
├── ui/                # UI framework and rendering

**Note**: Module name is `windows/` (directory). Dependency `windows-sys` (crate) used for low-level FFI where `windows` crate lacks APIs.
```

---

## Summary of Edits

| Issue | File(s) | Lines Changed | Complexity | Priority |
|-------|---------|---------------|------------|----------|
| **F1** (Array size) | tasks.md, plan.md | 10+ | Low | **CRITICAL** |
| **N3** (Mica/Acrylic) | tasks.md | 8 new tasks | Medium | **CRITICAL** |
| **F2** (Startup bench) | tasks.md, plan.md | 9 new tasks + docs | Medium | **CRITICAL** |
| **G1** (DPI v2) | tasks.md | 8 new tasks | Medium | **CRITICAL** |
| **A1** (Data ownership) | tasks.md, plan.md | 7 new tasks + diagram | High | **CRITICAL** |
| **N1** (Naming) | constitution.md | 1 clarification | Low | LOW |

**Total New Tasks**: 32  
**Total Documentation Updates**: 6 sections  
**Estimated Edit Time**: 30-45 minutes

---

## Validation After Edits

After applying all edits, run these commands to verify:

```powershell
# 1. Verify all tasks parse correctly
grep -n "^\- \[ \]" specs/001-native-task-manager/tasks.md | wc -l
# Should show 430+ tasks (400 original + 32 new)

# 2. Verify no duplicate task IDs
grep -o "T[0-9]\+" specs/001-native-task-manager/tasks.md | sort | uniq -d
# Should return empty (no duplicates)

# 3. Verify all capacity values updated
grep "1024" specs/001-native-task-manager/tasks.md
# Should return 0 results (all changed to 2048)

# 4. Re-run analysis to verify fixes
/speckit.analyze
```

Expected analysis results after fixes:
- **CRITICAL issues**: 5 → 0 ✅
- **Requirements coverage**: 41% → 65% ✅
- **Success criteria validation**: 13% → 40% ✅

---

## Application Order

Apply edits in this order to minimize merge conflicts:

1. **Edit 12** (constitution naming) - Independent, low risk
2. **Edit 1-3** (array sizing) - Low complexity, foundational
3. **Edit 4-5** (Mica/Acrylic tasks) - Phase 2 tasks
4. **Edit 8-9** (DPI v2 tasks) - Phase 2 tasks  
5. **Edit 6-7** (startup benchmarks) - Phase 3 tasks
6. **Edit 10-11** (data ownership) - Phase 3 foundational
7. **Verify**: Run analysis and validation commands

---

## Next Steps After Applying Edits

1. ✅ Commit changes: `git commit -m "Fix CRITICAL issues: array capacity, Mica/Acrylic, startup benchmarks, DPI v2, data ownership"`
2. ✅ Re-run analysis: `/speckit.analyze` to verify improvements
3. ✅ Review `ARCHITECTURE-CLARIFICATION.md` with team
4. ✅ Review `ANALYSIS-REMEDIATION-TASKS.md` for additional HIGH priority tasks
5. ✅ Begin Phase 1 implementation (safe to proceed after CRITICAL fixes)

---

**Document Status**: ✅ READY TO APPLY  
**Estimated Application Time**: 30-45 minutes  
**Risk Level**: LOW (all changes are additions or clarifications, no deletions)  
**Testing**: Validation commands provided, analysis re-run required

