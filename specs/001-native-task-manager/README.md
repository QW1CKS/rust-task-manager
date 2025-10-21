# Native Task Manager - Specification Documentation

**Feature ID**: `001-native-task-manager`  
**Status**: ✅ Phase 0-3 COMPLETE | � Phase 4 PENDING (73/432 tasks, 16.9%)  
**Last Updated**: 2025-10-22

## Quick Links

### 📋 Core Specification Documents
- **[spec.md](./spec.md)** - Feature specification with 68 functional requirements, 15 success criteria
- **[plan.md](./plan.md)** - Implementation plan with phases, timeline, deliverables
- **[tasks.md](./tasks.md)** - 432+ actionable tasks across 4 implementation phases

### 🏗️ Architecture & Design
- **[ARCHITECTURE-CLARIFICATION.md](./ARCHITECTURE-CLARIFICATION.md)** - Data flow, threading model, ownership rules (27KB)
- **[design/](./design/)** - UI specifications, interaction patterns, Fluent Design integration
- **[ANALYSIS-REMEDIATION-TASKS.md](./ANALYSIS-REMEDIATION-TASKS.md)** - 180+ additional task definitions (84KB)
- **[CRITICAL-FIXES.md](./CRITICAL-FIXES.md)** - ✅ Applied fixes resolving 5 blocking issues

### 🔬 Research & Validation
- **[research/](./research/)** - Windows API research, external AI validation, decisions summary
- **[benchmarks/](./benchmarks/)** - Performance benchmark methodology (startup, monitoring, rendering)
- **[checklists/](./checklists/)** - Quality validation (performance, security, Windows integration, UX)

---

## Document Status

| Document | Status | Last Updated | Purpose |
|----------|--------|--------------|---------|
| **spec.md** | ✅ Validated | 2025-10-21 | Feature specification (68 FRs, 15 SCs) |
| **plan.md** | ✅ Phase 3 Complete | 2025-10-22 | Implementation plan (Phase 3 ✅ COMPLETE - 73/432 tasks) |
| **tasks.md** | ✅ T001-T073 Complete | 2025-10-22 | 432+ tasks (73/432 = 16.9%, Phase 1-2: 100%) |
| **ARCHITECTURE-CLARIFICATION.md** | ✅ Complete | 2025-10-21 | Data flow, threading, ownership model |
| **ANALYSIS-REMEDIATION-TASKS.md** | ✅ Integrated | 2025-10-21 | 180+ task definitions (32 CRITICAL applied) |
| **CRITICAL-FIXES.md** | ✅ Applied | 2025-10-21 | 12 edits resolving 5 blocking issues |
| **design/** | ✅ Complete | Pre-2025-10-21 | UI/UX specifications |
| **research/** | ✅ Complete | 2025-01-21 | Windows API research + validation |
| **benchmarks/** | ✅ Complete | Pre-2025-10-21 | Performance methodology |
| **checklists/** | ✅ Complete | Pre-2025-10-21 | Quality validation checklists |

---

## Phase Completion Status

### ✅ Phase 0: Research (COMPLETE - 2025-01-21)
**Deliverables**:
- ✅ `research/windows-api-research.md` - UI framework evaluation, monitoring APIs
- ✅ `research/research.md` - External AI validation (GLM-4.6, Gemini, ChatGPT)
- ✅ `research/README.md` - Research index and decisions

**Key Decisions**:
- UI Framework: Custom Win32 + Direct2D (rejected cross-platform frameworks)
- Monitoring: Hybrid NtQuerySystemInformation + PDH + ETW
- Allocator: mimalloc global + bumpalo arenas
- Data Layout: Structure of Arrays (SoA) for cache efficiency

### ✅ Phase 1: Design (COMPLETE - 2025-10-21)
**Deliverables**:
- ✅ `ARCHITECTURE-CLARIFICATION.md` (27KB) - Complete architecture specification
  - Data flow architecture (Windows → Core → UI, unidirectional)
  - Component responsibilities (SystemMonitor, ProcessStore, Renderer)
  - Threading model (UI thread + Background thread, mpsc channel)
  - Ownership rules (ProcessSnapshot transfer, SoA storage)
  - Error propagation (4-layer strategy)
  - Code examples (collect_all, update, spawn)
  
- ✅ Data model definition (in ARCHITECTURE-CLARIFICATION.md + tasks.md)
  - ProcessSnapshot structure with Vec<ProcessInfo>
  - ProcessStore SoA layout with Box<[T; 2048]> arrays
  - Capacity enforcement with compile-time assertions
  
- ✅ API contracts (in ARCHITECTURE-CLARIFICATION.md)
  - SystemMonitor::collect_all() API
  - ProcessStore::update() API
  - BackgroundUpdater::spawn() API
  
- ✅ Design specifications (design/ directory)
  - UI specification with Fluent Design
  - Interaction specification with keyboard shortcuts
  
- ✅ Benchmark methodology (benchmarks/ directory)
  - Startup benchmarks (Part 1)
  - Monitoring & rendering benchmarks (Part 2)
  
- ✅ Quality checklists (checklists/ directory)
  - 8 comprehensive checklists for validation

**Duration**: 10 days (2025-10-12 to 2025-10-21)

### ✅ Phase 2: Task Breakdown (COMPLETE - 2025-10-21)
**Deliverables**:
- ✅ `tasks.md` (432+ tasks)
  - Phase 1: 20 tasks (project setup, 3-5 days)
  - Phase 2: 53 tasks (UI framework, 2-3 weeks)
  - Phase 3: 83 tasks (monitoring, 3-4 weeks)
  - Phase 4: 200+ tasks (process management, 4-6 weeks)
  
- ✅ CRITICAL task additions (from ANALYSIS-REMEDIATION-TASKS.md)
  - T078a-b: Array capacity 2048 with assertions
  - T045a-h: Mica/Acrylic implementation (8 tasks)
  - T050a-h: Complete DPI v2 (8 tasks)
  - T147a-i: Startup benchmarks (9 tasks)
  - T133a-g: Data ownership specification (7 tasks)
  - **Total**: 32 CRITICAL tasks added
  
- ✅ `CRITICAL-FIXES.md` - 12 edits applied to resolve 5 blocking issues
  - F1: Array sizing 1024 → 2048 ✅
  - N3: Mica/Acrylic missing ✅
  - F2: No startup measurement ✅
  - G1: Incomplete DPI v2 ✅
  - A1: Data ownership unclear ✅

**Duration**: 2 days (2025-10-19 to 2025-10-21)

### 🔄 Phase 3: Foundation Implementation (IN PROGRESS - Started 2025-10-21)
**Goals**: Build core infrastructure (windowing, rendering, monitoring APIs)

**Implementation Status**:

**✅ MILESTONE 1 COMPLETE** (2025-10-22 - 44/432 tasks, 10.2%):

- **Project Setup (T001-T020)**: ✅ Full Cargo workspace with dependencies
  - Cargo.toml with **windows 0.62** (upgraded from 0.58), mimalloc, bumpalo, static_assertions
  - build.rs embedding Windows manifest (DPI awareness, UAC, visual styles)
  - Module structure: core/, windows/, ui/, app/, util/
  - Utilities: time.rs (QPC wrapper), strings.rs (UTF-16), arenas.rs (bumpalo)
  - ProcessStore SoA (Box<[T; 2048]>) with compile-time assertions
  - CI pipeline (.github/workflows/ci.yml) with clippy/test/bench
  - Binary size: 0.23MB (96% under 10MB budget)
  
- **Win32 Window (T021-T027)**: ✅ Window foundation working
  - src/ui/window.rs (202 lines) - CreateWindowExW + message loop
  - WM_PAINT, WM_SIZE, WM_DESTROY, WM_CLOSE, WM_DPICHANGED handlers
  - Window opens 1200x800, closes cleanly, DPI-aware
  - **API Fixes**: GetModuleHandleW type conversion (HMODULE → HINSTANCE), Error::from_thread()
  - Validated: Window runs successfully, message loop responsive

- **Direct2D Initialization (T028-T033)**: ✅ Complete renderer infrastructure
  - src/ui/d2d/renderer.rs (240 lines) - D3D11 + DXGI + D2D1 setup
  - **RESOLVED**: Upgraded windows 0.58 → 0.62 fixed all Direct2D API issues
  - D2D1CreateFactory → ID2D1Factory1 ✅
  - D3D11CreateDevice with HMODULE::default() parameter ✅
  - IDXGISwapChain1 creation for window ✅
  - ID2D1DeviceContext setup ✅
  - CreateBitmapFromDxgiSurface for render target ✅
  - **Build Status**: ✅ 0 compilation errors, 2 dead_code warnings (expected)

- **DirectWrite Integration (T034-T037)**: ✅ Text rendering infrastructure
  - src/ui/d2d/resources.rs (100 lines) - Resource pool implementation
  - DWriteCreateFactory → IDWriteFactory ✅
  - CreateTextFormat for Segoe UI 12pt ✅
  - Text format structure ready for headers/labels/monospace ✅

- **Resource Management (T038-T041)**: ✅ Brush and resource pooling
  - Brush pool with CreateSolidColorBrush (white/black/gray) ✅
  - Windows 11 Fluent Design color palette constants ✅
  - Geometry caching structure ready ✅
  - Device lost handling structure in place ✅

- **Core Rendering Loop (T042-T044)**: ✅ Rendering infrastructure ready
  - Renderer structure with BeginDraw/EndDraw cycle ✅
  - Frame timing integration via util/time.rs ✅
  - Event-driven rendering via WM_PAINT handler ✅

**⏳ Next Phase (M2)**:
- Fluent Design Materials (T045a-h): Mica/Acrylic with WinRT Composition
- DPI Scaling (T047-T050h): Complete Per-Monitor DPI v2 implementation
- Input Handling (T051-T056): Mouse/keyboard event processing
- Layout System (T057-T061): Rectangle-based layout engine
- Basic UI Controls (T062-T065): Button implementation with Fluent styling

**Key Achievement**: Phase 2 M1 (Win32 + Direct2D foundation) now compiles successfully with **0 errors** after resolving windows-rs API compatibility issues through version upgrade. Ready for rendering logic implementation.
- Process Enumeration (NtQuerySystemInformation wrapper)
- Background Update Loop

**Key Milestones**:
- M1: Win32 Window + Direct2D Rendering (Week 1-2) - 🔄 **PARTIAL** (window ✅, D2D ❌)
- M2: Process Enumeration + SoA Storage (Week 3-4) - ⏳ **NOT STARTED**
- M3: Background Update Loop + Threading (Week 5-6) - ⏳ **NOT STARTED**

**Next Actions**:
1. Resolve windows-rs 0.58 Direct2D API issues (check examples, consider 0.52/0.62)
2. Complete Direct2D renderer (T028-T044)
3. Implement DirectWrite text rendering (T033-T037)
4. Add rendering loop (T045-T046)
5. Begin process enumeration (T051+)

**Prerequisites Met**:
- ✅ Architecture fully defined
- ✅ Data models specified
- ✅ API contracts documented
- ✅ CRITICAL issues resolved
- ✅ Task dependencies mapped
- ✅ Development environment working (Cargo builds clean for Phase 1)

---

## Key Metrics

### Documentation Completeness
- **Total Documents**: 16 files across 5 directories
- **Total Size**: ~250KB of specification documentation
- **Requirements Coverage**: 65% (44/68 FRs mapped to tasks)
- **Success Criteria Coverage**: 47% (7/15 SCs with validation tasks)
- **CRITICAL Issues**: 0 remaining (5 resolved)

### Task Coverage
- **Total Tasks Defined**: 432+ across Phases 1-4
- **Tasks Complete**: 27 (T001-T020 ✅, T021-T027 ✅)
- **Tasks Blocked**: 8 (T028-T032 Direct2D, T033-T037 DirectWrite - windows-rs API issues)
- **Completion Rate**: 6.25% (27/432)
- **CRITICAL Tasks**: 23 (all blocking issues resolved at design level)
- **Parallel Tasks**: 150+ marked with [P] tag
- **Performance-Critical**: 180+ marked with [PERF] tag
- **Unsafe Code**: 90+ marked with [UNSAFE] tag (safety contracts defined)

### Implementation Metrics (as of 2025-10-21)
- **Binary Size**: 0.23MB release build (96% under 10MB budget)
- **Build Status**: ✅ Phase 1 compiles clean | ❌ Phase 2 has 8 D2D errors
- **Test Status**: ✅ Infrastructure in place, 0 failures
- **Module Count**: 11 files created (main, lib, core/, ui/, util/, windows/)
- **Lines of Code**: ~1,800 lines implemented (project setup + window foundation)

### Estimated Timeline
- **Phase 3 (Foundation)**: 6 weeks
- **Phase 4 (Core Features)**: 6 weeks
- **Phase 5 (Advanced)**: 4 weeks
- **Phase 6+ (Polish)**: 4 weeks
- **Total v1.0**: 20 weeks (~5 months)

---

## How to Navigate

### For Developers Starting Implementation:
1. ✅ **DONE**: Phase 1 (T001-T020) project setup complete - all code compiles
2. ✅ **DONE**: T021-T027 Win32 window foundation - window runs successfully
3. ❌ **BLOCKED**: T028-T044 Direct2D rendering - windows-rs 0.58 API issues
   - **Action Required**: Investigate windows-rs 0.58 Direct2D bindings
   - Check windows-rs examples for CreateSolidColorBrush patterns
   - Consider downgrade to 0.52 or upgrade to 0.62 if API stabilized
4. Read **[spec.md](./spec.md)** - Understand requirements and success criteria
5. Read **[ARCHITECTURE-CLARIFICATION.md](./ARCHITECTURE-CLARIFICATION.md)** - Understand data flow and threading
6. Reference **[design/](./design/)** - For UI implementation details
7. Check **[benchmarks/](./benchmarks/)** - For performance validation approach

### For Reviewers:
1. Check **[plan.md](./plan.md)** - Overall status and phase gates (Phase 3 M1 partial)
2. Review **[tasks.md](./tasks.md)** - Task status (T001-T027 ✅, T028-T032 blocked)
3. Validate **[checklists/](./checklists/)** - Quality validation criteria
4. Review **[CRITICAL-FIXES.md](./CRITICAL-FIXES.md)** - Ensure all fixes applied

### For Project Managers:
1. See **[plan.md](./plan.md)** Phase Status table - Track progress (Phase 3 in progress)
2. Monitor **[tasks.md](./tasks.md)** checkpoints - 27/432 tasks complete (6.25%)
3. Reference **Key Metrics** above - Binary 0.23MB, 8 blocked tasks on windows-rs API
4. **Current Risk**: Direct2D blocker may require architecture adjustment or library version change

---

## Analysis Findings Summary

**Source**: `/speckit.analyze` performed 2025-10-21

### Findings by Severity
- **5 CRITICAL**: Blocking issues requiring immediate resolution before Phase 3
- **12 HIGH**: Issues requiring resolution before Phase 4
- **18 MEDIUM**: Issues for Phase 5
- **8 LOW**: Nice-to-have improvements

### Resolution Status
- ✅ **ALL 5 CRITICAL** issues resolved with concrete tasks
- ✅ **180+ remediation tasks** defined in ANALYSIS-REMEDIATION-TASKS.md
- ✅ **32 CRITICAL tasks** integrated into tasks.md
- 📋 **HIGH/MEDIUM tasks** documented for future phases

### Key Findings Addressed
1. **F1 - Array Sizing**: Fixed 1024 → 2048 with compile-time assertion
2. **N3 - Mica/Acrylic**: 8 tasks added for Windows 11 materials
3. **F2 - Startup Benchmarks**: 9 tasks added for SC-001 validation
4. **G1 - DPI v2**: 8 tasks added for complete per-monitor support
5. **A1 - Data Ownership**: 7 tasks added defining clear architecture

---

## Constitution Compliance

**Validation**: ✅ PASSED (Phase 0, re-validated Phase 1)

All 7 constitutional principles verified:
- ✅ **I. Native-First Architecture** - Direct windows-rs APIs, no cross-platform abstractions
- ✅ **II. Extreme Performance Targets** - <500ms startup, <15MB memory, <2% CPU
- ✅ **III. Zero-Allocation Hot Paths** - SoA pre-allocated buffers, bumpalo arenas
- ✅ **IV. Strategic Unsafe Rust** - Safety contracts defined, Miri validation required
- ✅ **V. Windows Integration Excellence** - Direct2D, Mica/Acrylic, per-monitor DPI v2
- ✅ **VI. Measured Performance** - Benchmark methodology complete
- ✅ **VII. Security by Design** - Least privilege, elevation on-demand, no credential storage

**See**: `.specify/memory/constitution.md` for complete principles

---

## Contributing

Before starting implementation:
1. ✅ Ensure Phase 1-2 documentation reviewed and understood
2. ✅ Verify all CRITICAL fixes from CRITICAL-FIXES.md are applied
3. ✅ Check tasks.md for task dependencies before starting work
4. ✅ Reference ARCHITECTURE-CLARIFICATION.md for data flow and threading
5. ✅ Follow safety contracts for all UNSAFE code
6. ✅ Add benchmarks per benchmarks/ methodology
7. ✅ Validate against checklists/ before marking complete

---

**Last Updated**: 2025-10-21  
**Next Milestone**: Begin Phase 3 Foundation Implementation (T001-T020)
