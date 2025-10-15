# Implementation Plan: Modern Windows Task Manager

**Branch**: `001-build-a-modern` | **Date**: 2025-10-15 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-build-a-modern/spec.md`

## Summary

Build a modern, lightweight Windows task manager application that provides real-time system monitoring and process management capabilities. The application will display system specifications, performance metrics (CPU, memory, disk I/O, network), and allow users to view, sort, filter, and terminate processes. The implementation uses Tauri 2.x for the desktop framework with Rust backend for system operations and TypeScript/Vite frontend for the UI, targeting < 2 second startup, < 50MB memory footprint, and < 5% idle CPU usage.

## Technical Context

**Language/Version**: Rust 1.70+ (2021 edition) for backend, TypeScript 5.x with strict mode for frontend  
**Primary Dependencies**: 
- Backend: Tauri 2.x, sysinfo 0.32+, serde, serde_json, thiserror (error handling)
- Frontend: Vite (build tool), Chart.js (visualizations), vanilla TypeScript (no framework)
- Dev Tools: cargo clippy (Rust linting), rustfmt (formatting), ESLint + Prettier (TypeScript)

**Storage**: Local file system for user preferences (JSON config file), in-memory only for performance history (60-second rolling buffer)  
**Testing**: cargo test (Rust unit/integration tests), Tauri test harness (command integration tests), manual frontend testing (no heavy test framework for MVP)  
**Target Platform**: Windows 10 (1809+) and Windows 11, x64 architecture (ARM64 future consideration)  
**Project Type**: Desktop application (Tauri hybrid architecture: Rust backend + web frontend in single binary)  
**Performance Goals**: 
- < 2 second cold start to interactive UI
- < 50MB memory footprint at idle
- < 5% CPU usage when idle (between 1-2 second refresh cycles)
- 60 FPS UI animations and smooth scrolling
- Handle 500+ processes without lag

**Constraints**: 
- Must use native Windows APIs for optimal performance
- No elevated privileges by default (only on-demand for process termination)
- Single window instance (no multi-window support)
- English only (no i18n in MVP)
- No long-term historical data persistence (future enhancement)

**Scale/Scope**: 
- Single-user desktop application
- ~5,000 lines of Rust code (estimated)
- ~2,000 lines of TypeScript code (estimated)
- 6 Tauri commands (system_info, performance_data, processes, kill_process, process_details, plus theme preference)
- 5 core UI screens/sections (system info, performance graphs, process list, process details, settings)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Principle I: Type Safety & Error Handling
- ✅ **PASS**: All Tauri commands return `Result<T, String>` or custom error enums
- ✅ **PASS**: TypeScript strict mode enabled (no implicit `any`)
- ✅ **PASS**: Custom error types using `thiserror` for domain errors
- ✅ **PASS**: No `.unwrap()` in production paths (test code only)
- ✅ **PASS**: Validation at API boundaries (Tauri command inputs validated before processing)

### Principle II: Performance-First Architecture
- ✅ **PASS**: Target < 2s startup explicitly documented in spec (SC-001)
- ✅ **PASS**: Target < 50MB memory footprint at idle (SC-006)
- ✅ **PASS**: Target < 5% CPU at idle (SC-007)
- ✅ **PASS**: 1-2 second refresh interval specified (FR-002)
- ✅ **PASS**: Caching strategy defined (100ms cache for performance data)
- ✅ **PASS**: Virtualization for process list when > 100 items
- ✅ **PASS**: Async Tauri commands to avoid UI blocking

### Principle III: Windows Platform Optimization
- ✅ **PASS**: sysinfo crate provides Windows-optimized APIs
- ✅ **PASS**: Windows-specific UAC handling specified (FR-015, FR-022)
- ✅ **PASS**: Critical Windows process protection (FR-023)
- ✅ **PASS**: System-native fonts (Segoe UI) specified (FR-018)
- ✅ **PASS**: Windows keyboard shortcuts planned (FR-016)
- ⚠️ **REVIEW**: Consider `windows` crate for deeper integration beyond sysinfo

### Principle IV: Modern Minimalist UI/UX
- ✅ **PASS**: Dark mode default with light mode toggle (FR-011)
- ✅ **PASS**: Color palette specified (#1a1a1a, #2d2d2d, #3b82f6) in constitution
- ✅ **PASS**: Segoe UI fonts specified (FR-018)
- ✅ **PASS**: CSS Grid/Flexbox for responsive layouts
- ✅ **PASS**: Hardware-accelerated animations (transform, opacity)
- ✅ **PASS**: Animation duration 150-300ms (FR-019)
- ✅ **PASS**: Loading states specified (FR-024)
- ✅ **PASS**: Error states specified (FR-021)

### Principle V: Security & Safe Operations
- ✅ **PASS**: Process termination confirmation dialog (FR-006)
- ✅ **PASS**: Critical process warning (FR-023)
- ✅ **PASS**: UAC elevation on-demand only (FR-015)
- ✅ **PASS**: Privilege denial handling (FR-022)
- ✅ **PASS**: No sensitive data logging (constitution requirement)
- ✅ **PASS**: Process disappearance gracefully handled (FR-014)

### Principle VI: Test-Driven Development
- ✅ **PASS**: Unit tests required for Rust backend (constitution mandates >80% coverage)
- ✅ **PASS**: Integration tests for Tauri commands (constitution requirement)
- ✅ **PASS**: Test file naming convention established (`[module]_tests.rs`)
- ✅ **PASS**: Mock system APIs using traits for testability
- ⚠️ **REVIEW**: CI/CD pipeline not yet configured (add to tasks)
- ⚠️ **REVIEW**: Frontend testing strategy needs detail (constitution mentions critical interactions)

### Principle VII: Documentation Excellence
- ✅ **PASS**: Rust doc comments required for public APIs (`///`)
- ✅ **PASS**: JSDoc for TypeScript exports
- ✅ **PASS**: Inline comments for complex logic
- ✅ **PASS**: Tooltips for technical terms (FR-017)
- ✅ **PASS**: README maintenance required
- ⚠️ **REVIEW**: CHANGELOG.md not yet established (add to project setup)

**Overall Status**: ✅ **PASS WITH MINOR REVIEWS**

**Action Items**:
1. Consider adding `windows` crate for deeper Windows API integration beyond sysinfo
2. Define CI/CD pipeline configuration (GitHub Actions) in tasks phase
3. Specify frontend test strategy for critical user interactions
4. Initialize CHANGELOG.md during project setup

## Project Structure

### Documentation (this feature)

```
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
rust-task-manager/
├── src/                          # Frontend source (TypeScript + Vite)
│   ├── main.ts                   # Application entry point, Tauri command bindings
│   ├── style.css                 # Global styles, theming, CSS custom properties
│   ├── types/                    # TypeScript type definitions
│   │   ├── system.ts             # System info types
│   │   ├── process.ts            # Process types
│   │   └── performance.ts        # Performance metric types
│   ├── services/                 # Frontend service layer
│   │   ├── tauri.ts              # Tauri command wrappers
│   │   ├── performance.ts        # Performance data polling/caching
│   │   └── theme.ts              # Theme management
│   ├── components/               # UI components (vanilla TS)
│   │   ├── SystemInfo.ts         # System specifications display
│   │   ├── PerformanceCharts.ts  # Chart.js integration
│   │   ├── ProcessList.ts        # Process table with sorting/filtering
│   │   ├── ProcessDetails.ts     # Detailed process modal
│   │   └── ThemeToggle.ts        # Dark/light mode switch
│   └── utils/                    # Utility functions
│       ├── formatters.ts         # Data formatting (MB, %, timestamps)
│       └── virtualization.ts     # Process list virtualization logic
│
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # Tauri app setup, command registration
│   │   ├── lib.rs                # Library root (if needed for testing)
│   │   ├── commands/             # Tauri command implementations
│   │   │   ├── mod.rs
│   │   │   ├── system_info.rs    # get_system_info command
│   │   │   ├── performance.rs    # get_performance_data command
│   │   │   ├── processes.rs      # get_processes command
│   │   │   ├── process_ops.rs    # kill_process, get_process_details
│   │   │   └── preferences.rs    # Load/save user preferences
│   │   ├── models/               # Data structures
│   │   │   ├── mod.rs
│   │   │   ├── system.rs         # SystemInfo struct
│   │   │   ├── process.rs        # ProcessInfo struct, ProcessStatus enum
│   │   │   ├── performance.rs    # PerformanceMetrics struct
│   │   │   └── preferences.rs    # UserPreferences struct
│   │   ├── services/             # Business logic
│   │   │   ├── mod.rs
│   │   │   ├── system_monitor.rs # sysinfo integration
│   │   │   ├── process_manager.rs# Process operations with safety checks
│   │   │   └── config.rs         # Preferences persistence
│   │   ├── errors/               # Custom error types
│   │   │   ├── mod.rs
│   │   │   └── app_error.rs      # AppError enum with thiserror
│   │   └── utils/                # Helper functions
│   │       ├── mod.rs
│   │       └── windows.rs        # Windows-specific utilities (UAC, critical process list)
│   ├── Cargo.toml                # Rust dependencies
│   ├── tauri.conf.json           # Tauri configuration
│   └── build.rs                  # Build script
│
├── tests/                        # Rust integration tests
│   ├── common/
│   │   └── mod.rs                # Shared test utilities
│   ├── commands_test.rs          # Tauri command contract tests
│   └── system_monitor_test.rs   # System monitoring integration tests
│
├── index.html                    # HTML entry point
├── package.json                  # npm dependencies and scripts
├── tsconfig.json                 # TypeScript configuration
├── vite.config.ts                # Vite build configuration
├── .eslintrc.json                # ESLint configuration
├── .prettierrc                   # Prettier configuration
├── CHANGELOG.md                  # Version history
└── README.md                     # Project documentation
```

**Structure Decision**: Tauri desktop application structure with clear separation between Rust backend (`src-tauri/`) and TypeScript frontend (`src/`). This follows the standard Tauri project layout and aligns with the constitution's principle of clean architecture. The Rust backend is organized into commands (Tauri API layer), models (data structures), services (business logic), and errors (type-safe error handling). The frontend uses a component-based vanilla TypeScript approach without a heavy framework, keeping the bundle size minimal for performance targets.

## Complexity Tracking

*No constitution violations requiring justification. All constitution checks passed or have minor review items that don't block implementation.*

## Phase 0: Research & Technical Decisions

### Research Topics

All technical decisions have been provided by the user and align with the specification. No research phase required as all "NEEDS CLARIFICATION" items were pre-resolved:

1. **Framework Choice**: Tauri 2.x ✅ (User-specified, appropriate for desktop app)
2. **Backend Language**: Rust 1.70+ ✅ (User-specified, performance requirement)
3. **System Monitoring Library**: sysinfo 0.32+ ✅ (User-specified, cross-platform with Windows optimization)
4. **Frontend Stack**: TypeScript + Vite + vanilla JS ✅ (User-specified, lightweight)
5. **Charting Library**: Chart.js ✅ (User-specified, mature and performant)
6. **Error Handling**: thiserror ✅ (Constitution requirement, user-specified)

### Key Technical Decisions

**Decision 1: Tauri vs Electron**
- **Chosen**: Tauri 2.x
- **Rationale**: Significantly smaller bundle size, better performance, Rust backend aligns with constitution's performance-first and type-safety principles
- **Alternatives Considered**: Electron (rejected: higher memory footprint conflicts with <50MB target)

**Decision 2: Vanilla TypeScript vs Framework (React/Vue)**
- **Chosen**: Vanilla TypeScript with component pattern
- **Rationale**: Minimal bundle size, no framework overhead, sufficient for UI complexity, aligns with <2s startup target
- **Alternatives Considered**: React (rejected: adds ~100KB+ to bundle, unnecessary for this scope)

**Decision 3: sysinfo vs Native Windows APIs**
- **Chosen**: sysinfo 0.32+ as primary, with option to augment with `windows` crate if needed
- **Rationale**: sysinfo provides cross-platform abstraction with Windows-optimized implementation, reduces code complexity
- **Alternatives Considered**: Pure WinAPI via `windows` crate (deferred: sysinfo sufficient for MVP, can enhance later)

**Decision 4: Process List Virtualization Strategy**
- **Chosen**: Virtual scrolling when > 100 items
- **Rationale**: Maintains 60 FPS scrolling with 500+ processes, aligns with performance constitution
- **Implementation**: Render only visible rows + buffer (±20 rows), reuse DOM nodes

**Decision 5: Performance Data Caching**
- **Chosen**: 100ms cache window for performance data
- **Rationale**: Prevents excessive Tauri IPC calls when multiple UI components request same data, balances freshness with performance
- **Trade-off**: Slight data staleness acceptable within 100ms window

**Decision 6: Theme Implementation**
- **Chosen**: CSS custom properties with class-based switching
- **Rationale**: Hardware-accelerated, no JS framework dependency, persisted via local JSON config
- **Implementation**: `<body class="dark-theme">` vs `<body class="light-theme">`

### Phase 0 Outputs

- ✅ **research.md**: Complete technical research document covering 16 topics
  - Framework choices (Tauri vs Electron)
  - Frontend stack (vanilla TypeScript vs frameworks)
  - System monitoring approach (sysinfo vs native APIs)
  - Architecture patterns (command layer + service layer)
  - Performance optimizations (caching, virtualization, async patterns)
  - Security considerations (UAC handling, critical process protection)
  - Testing strategy (unit, integration, manual)

## Phase 1: Design & Contracts

**Status**: ✅ **COMPLETE**

### Deliverables

1. **data-model.md**: ✅ Complete
   - 5 core entities defined: SystemInfo, PerformanceMetrics, ProcessInfo, UserPreferences, AppError
   - Both Rust and TypeScript definitions provided
   - Serialization examples and validation rules included
   - Data flow diagrams and lifecycle descriptions

2. **contracts/tauri-commands.md**: ✅ Complete
   - 7 Tauri commands fully specified:
     - `get_system_info` - Static system hardware/software info
     - `get_performance_data` - Real-time CPU/memory/disk/network metrics
     - `get_processes` - List all running processes
     - `kill_process` - Terminate process with safety checks
     - `get_process_details` - Detailed info for specific process
     - `get_preferences` - Load user preferences from disk
     - `save_preferences` - Save user preferences to disk
   - Request/response schemas, error cases, performance targets documented
   - IPC performance targets specified (< 50ms to < 500ms per command)
   - Security considerations and test cases included

3. **quickstart.md**: ✅ Complete
   - Prerequisites and installation instructions (Rust, Node.js, Visual Studio Build Tools, WebView2)
   - Development workflow (`npm run tauri:dev` for hot reload)
   - Production build process (`npm run tauri:build`)
   - Project structure explanation
   - Common commands and debugging tips
   - Performance validation procedures
   - Troubleshooting guide for common issues

4. **Agent Context Update**: ✅ Complete
   - Updated `.github/copilot-instructions.md` via `update-agent-context.ps1`
   - Added technology stack: Rust 1.70+, TypeScript 5.x, Tauri 2.x, sysinfo, Vite, Chart.js
   - Added project structure and commands

### Phase 1 Summary

All design artifacts complete and ready for implementation. The contracts define the API surface between Rust backend and TypeScript frontend, the data model provides type safety across the IPC boundary, and the quickstart guide enables developers to get up and running quickly. Agent context updated to reflect the complete technology stack.

## Phase 2: Tasks Generation

**Status**: ✅ **COMPLETE**

### Deliverable

**tasks.md**: ✅ Complete
- 118 total tasks organized by user story
- 8 setup tasks (Phase 1)
- 15 foundational tasks (Phase 2 - blocking prerequisites)
- 75 implementation tasks across 6 user stories (Phases 3-8)
- 18 polish and cross-cutting tasks (Phase 9)
- 45+ tasks marked [P] for parallel execution
- MVP scope identified: 52 tasks (Setup + Foundational + US1 + US2 + US3)

### Task Breakdown by User Story

- **User Story 1** (System Health - P1): 16 tasks
- **User Story 2** (Process List - P1): 13 tasks
- **User Story 3** (Terminate Process - P2): 14 tasks
- **User Story 4** (Trend Graphs - P2): 10 tasks
- **User Story 5** (Process Details - P3): 11 tasks
- **User Story 6** (Theme Toggle - P3): 13 tasks

### Task Organization

Tasks are organized to enable:
- **Independent Story Implementation**: Each user story has all its required tasks grouped together
- **Independent Testing**: Each story can be tested independently per acceptance scenarios
- **Parallel Execution**: 45+ tasks marked [P] can run in parallel (different files, no dependencies)
- **Incremental Delivery**: MVP (US1-US3) delivers core value, remaining stories add enhancements
- **Clear Dependencies**: Dependency graph shows US1 → US4, US2 → US3/US5, US6 independent

### Implementation Strategy

**MVP First** (Recommended):
1. Phase 1: Setup (8 tasks)
2. Phase 2: Foundational (15 tasks) - **BLOCKS all user stories**
3. Phase 3: User Story 1 (16 tasks) - System health display
4. Phase 4: User Story 2 (13 tasks) - Process list
5. Phase 5: User Story 3 (14 tasks) - Process termination
6. **Total MVP**: 66 tasks → Delivers complete diagnosis and resolution workflow

**Parallel Team** (With 3 developers):
- After Phase 2 completes, developers can work on independent stories:
  - Dev A: US1 → US4
  - Dev B: US2 → US3 → US5
  - Dev C: US6 → Polish

## Next Steps

Implementation is now fully planned and ready to begin. To start development:

1. **Review tasks.md**: Understand the 118-task breakdown and dependencies
2. **Setup environment**: Follow quickstart.md for prerequisites (Rust, Node.js, Tauri)
3. **Begin Phase 1**: Complete setup tasks (T001-T008)
4. **Complete Phase 2**: Foundational tasks (T009-T023) - **CRITICAL BLOCKER**
5. **Implement MVP**: User Stories 1-3 (T024-T066) for core functionality
6. **Enhance**: Add remaining user stories (T067-T100) as enhancements
7. **Polish**: Complete Phase 9 (T101-T118) for production readiness

Each task includes specific file paths and implementation details. Commit after each task or logical group.
