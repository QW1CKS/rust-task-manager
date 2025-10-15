# Tasks: Modern Windows Task Manager

**Input**: Design documents from `/specs/001-build-a-modern/`
**Prerequisites**: plan.md (complete), spec.md (complete), research.md (complete), data-model.md (complete), contracts/ (complete)

**Tests**: Manual testing only for MVP. No automated test tasks included per constitution (manual validation strategy).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions
- **Frontend**: `src/` (TypeScript + Vite)
- **Backend**: `src-tauri/src/` (Rust)
- **Tests**: `src-tauri/tests/` (Rust integration tests)
- **Specs**: `specs/001-build-a-modern/` (documentation)

---

## Phase 1: Setup (Shared Infrastructure) ✅ COMPLETE

**Purpose**: Project initialization and basic structure  
**Status**: ✅ Completed and validated on 2025-10-15  
**Documentation**: See `PHASE1_COMPLETE.md` for full validation results

- [x] T001 Verify Tauri 2.x project structure is initialized (index.html, package.json, src/, src-tauri/)
- [x] T002 [P] Configure Rust dependencies in `src-tauri/Cargo.toml` (sysinfo 0.32+, serde, serde_json, thiserror, tauri)
- [x] T003 [P] Configure frontend dependencies in `package.json` (TypeScript, Vite, Chart.js)
- [x] T004 [P] Configure TypeScript strict mode in `tsconfig.json`
- [x] T005 [P] Configure Tauri window settings in `src-tauri/tauri.conf.json` (title, size, theme)
- [x] T006 [P] Setup ESLint and Prettier for TypeScript in `.eslintrc.json` and `.prettierrc`
- [x] T007 [P] Configure Rust formatting with `rustfmt.toml`
- [x] T008 Create CHANGELOG.md with initial version 0.1.0

**Checkpoint**: ✅ Project structure validated, all tools configured and working
- ✅ cargo check: Compiles successfully
- ✅ cargo clippy: No warnings
- ✅ cargo fmt: All code formatted
- ✅ npm run lint: All TypeScript passes
- ✅ npm run format: All files formatted
- ✅ 137 npm packages installed

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T009 [P] Create AppError enum with thiserror in `src-tauri/src/error.rs` (ProcessNotFound, AccessDenied, CriticalProcessProtection, SystemInfoError, PerformanceError, TerminationFailed variants)
- [ ] T010 [P] Create SystemInfo struct in `src-tauri/src/models/system.rs` (os_name, os_version, cpu_model, cpu_architecture, cpu_cores, total_memory, hostname fields with serde Serialize)
- [ ] T011 [P] Create PerformanceMetrics struct in `src-tauri/src/models/performance.rs` (timestamp, cpu_usage_percent, cpu_per_core, memory_used, memory_total, memory_percent, disk_read_bps, disk_write_bps, network_upload_bps, network_download_bps with serde Serialize)
- [ ] T012 [P] Create ProcessStatus enum in `src-tauri/src/models/process.rs` (Running, Sleeping, Stopped, Other variants with serde Serialize)
- [ ] T013 Create ProcessInfo struct in `src-tauri/src/models/process.rs` (pid, name, exe_path, cmd_args, cpu_percent, memory_bytes, status, parent_pid, start_time, user fields with serde Serialize)
- [ ] T014 [P] Create UserPreferences struct in `src-tauri/src/models/preferences.rs` (theme, window, sort_column, sort_order fields with serde Serialize/Deserialize)
- [ ] T015 [P] Create TypeScript SystemInfo interface in `src/types/system.ts` matching Rust struct
- [ ] T016 [P] Create TypeScript PerformanceMetrics interface in `src/types/performance.ts` matching Rust struct
- [ ] T017 [P] Create TypeScript ProcessInfo and ProcessStatus types in `src/types/process.ts` matching Rust structs
- [ ] T018 [P] Create TypeScript UserPreferences interface in `src/types/preferences.ts` matching Rust struct
- [ ] T019 Initialize sysinfo System in `src-tauri/src/services/system_monitor.rs` with lazy_static or once_cell for global access
- [ ] T020 [P] Create critical process list constant in `src-tauri/src/utils/windows.rs` (csrss.exe, wininit.exe, services.exe, smss.exe, lsass.exe)
- [ ] T021 [P] Setup global CSS custom properties in `src/style.css` (dark theme colors: --bg-primary: #1a1a1a, --bg-secondary: #2d2d2d, --accent: #3b82f6, --text-primary: #ffffff, etc.)
- [ ] T022 [P] Setup light theme CSS custom properties in `src/style.css` with .light-theme class override
- [ ] T023 Create base HTML structure in `index.html` (header, system-info section, performance section, process-list section)

**Checkpoint**: Foundation ready - all models, types, and base infrastructure complete. User story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Quick System Health Check (Priority: P1) 🎯 MVP

**Goal**: Display system specifications and current performance metrics within 2 seconds of launch

**Independent Test**: Launch application and verify system info (OS, CPU, RAM, hostname) and performance metrics (CPU %, memory %, network, disk) are visible and updating every 1-2 seconds. No interaction required.

### Implementation for User Story 1

- [ ] T024 [P] [US1] Implement get_system_info Tauri command in `src-tauri/src/commands/system_info.rs` (queries sysinfo once, returns SystemInfo struct)
- [ ] T025 [P] [US1] Implement get_performance_data Tauri command in `src-tauri/src/commands/performance.rs` (refreshes system, collects metrics, returns PerformanceMetrics struct)
- [ ] T026 [US1] Register get_system_info and get_performance_data commands in `src-tauri/src/main.rs` tauri::Builder
- [ ] T027 [P] [US1] Create SystemInfoService in `src-tauri/src/services/system_monitor.rs` with collect_system_info() method (wraps sysinfo queries, handles errors → AppError)
- [ ] T028 [P] [US1] Create PerformanceService in `src-tauri/src/services/system_monitor.rs` with collect_performance_data() method (handles refresh, calculates deltas for disk/network)
- [ ] T029 [P] [US1] Create Tauri command wrapper in `src/services/tauri.ts` (invokeGetSystemInfo and invokeGetPerformanceData functions with proper typing)
- [ ] T030 [US1] Create SystemInfo UI component in `src/components/SystemInfo.ts` (vanilla TypeScript class, renders OS, CPU, RAM, hostname in grid layout)
- [ ] T031 [US1] Create PerformanceMetrics UI component in `src/components/PerformanceMetrics.ts` (displays current CPU %, memory %, disk I/O, network speeds with color coding)
- [ ] T032 [US1] Implement color coding logic in `src/utils/formatters.ts` (getUsageColor function: green <50%, yellow 50-80%, red >80%)
- [ ] T033 [US1] Create performance data polling service in `src/services/performance.ts` (polls every 1-2 seconds with 100ms cache, handles errors per FR-021)
- [ ] T034 [US1] Implement loading state UI in `src/main.ts` (skeleton layout with spinner and "Loading system information..." text per FR-024)
- [ ] T035 [US1] Wire up SystemInfo component in `src/main.ts` (call invokeGetSystemInfo on startup, render result, handle errors)
- [ ] T036 [US1] Wire up PerformanceMetrics component in `src/main.ts` (start polling on startup, update UI every 1-2 seconds, show "Error" if metrics fail per FR-021)
- [ ] T037 [US1] Add data formatting utilities in `src/utils/formatters.ts` (formatBytes for MB/GB, formatPercent, formatSpeed for KB/s, MB/s, GB/s)
- [ ] T038 [US1] Style system info section in `src/style.css` (grid layout, card styling, responsive font sizes)
- [ ] T039 [US1] Style performance metrics section in `src/style.css` (flex layout, color-coded indicators, smooth transitions)

**Checkpoint**: User Story 1 complete. Application should launch in <2 seconds and display real-time system health metrics. Test independently.

---

## Phase 4: User Story 2 - Identify Resource-Heavy Processes (Priority: P1)

**Goal**: Display all running processes with sorting and filtering capabilities

**Independent Test**: Open application, view process list showing all processes with PID, name, CPU, memory, status. Click column headers to sort, use search box to filter. Verify smooth scrolling with 500+ processes.

### Implementation for User Story 2

- [ ] T040 [US2] Implement get_processes Tauri command in `src-tauri/src/commands/processes.rs` (refreshes processes, iterates sysinfo processes, returns Vec<ProcessInfo>)
- [ ] T041 [US2] Register get_processes command in `src-tauri/src/main.rs`
- [ ] T042 [US2] Create ProcessService in `src-tauri/src/services/process_manager.rs` with collect_processes() method (handles sysinfo process iteration, maps to ProcessInfo, handles access denied gracefully)
- [ ] T043 [P] [US2] Create Tauri command wrapper in `src/services/tauri.ts` (invokeGetProcesses function)
- [ ] T044 [US2] Create ProcessList UI component in `src/components/ProcessList.ts` (table rendering, virtualization when >100 processes per FR-013)
- [ ] T045 [US2] Implement table sorting logic in `src/components/ProcessList.ts` (click column header to toggle asc/desc, visual sort indicators)
- [ ] T046 [US2] Implement search/filter logic in `src/components/ProcessList.ts` (real-time filtering on process name, debounced 300ms)
- [ ] T047 [US2] Create process list polling service in `src/services/performance.ts` (polls every 1-2 seconds, updates table smoothly per FR-002)
- [ ] T048 [US2] Implement virtualization utility in `src/utils/virtualization.ts` (render only visible rows + buffer of ±20 rows, reuse DOM nodes)
- [ ] T049 [US2] Wire up ProcessList component in `src/main.ts` (start polling, render table, handle sort/filter interactions)
- [ ] T050 [US2] Style process list table in `src/style.css` (table layout, header styling, row hover effects, monospace fonts for numbers)
- [ ] T051 [US2] Add search box UI in `index.html` and wire to filter function
- [ ] T052 [US2] Handle process disappearance gracefully in `src/services/performance.ts` (remove from list without errors per FR-014)

**Checkpoint**: User Stories 1 AND 2 complete. Application displays system health AND full process list with sorting and filtering. Test independently.

---

## Phase 5: User Story 3 - Terminate Unresponsive Process (Priority: P2)

**Goal**: Allow users to terminate processes with confirmation and privilege handling

**Independent Test**: Select any process, right-click → "End Process", confirm dialog, verify process terminates within 2 seconds. Test with elevated process to verify UAC prompt.

### Implementation for User Story 3

- [ ] T053 [US3] Implement kill_process Tauri command in `src-tauri/src/commands/process_ops.rs` (validates PID, checks critical process list, calls process.kill(), handles UAC elevation)
- [ ] T054 [US3] Register kill_process command in `src-tauri/src/main.rs`
- [ ] T055 [US3] Create ProcessManager::terminate_process() method in `src-tauri/src/services/process_manager.rs` (safety checks: critical process per FR-023, process exists, returns Result<(), AppError>)
- [ ] T056 [US3] Implement critical process check in `src-tauri/src/services/process_manager.rs` (compare against critical list from utils/windows.rs, return CriticalProcessProtection error if match)
- [ ] T057 [P] [US3] Create Tauri command wrapper in `src/services/tauri.ts` (invokeKillProcess function with error handling)
- [ ] T058 [US3] Create confirmation dialog UI component in `src/components/ConfirmDialog.ts` (modal overlay, process name/PID, "Data loss" warning per FR-006)
- [ ] T059 [US3] Create critical process warning dialog in `src/components/ConfirmDialog.ts` (strong warning per FR-023: "WARNING: [name] is critical...", "I Understand, Terminate" button, focus on Cancel)
- [ ] T060 [US3] Create UAC denial retry dialog in `src/components/ConfirmDialog.ts` (informative message per FR-022: "Cannot terminate [name]: Admin privileges required...", Retry/Cancel buttons)
- [ ] T061 [US3] Add right-click context menu to ProcessList in `src/components/ProcessList.ts` ("End Process" option)
- [ ] T062 [US3] Wire up process termination flow in `src/components/ProcessList.ts` (right-click → show confirmation → call invokeKillProcess → handle errors → refresh process list)
- [ ] T063 [US3] Handle UAC elevation in Tauri command (Windows-specific, may trigger system UAC prompt automatically)
- [ ] T064 [US3] Handle error cases in `src/components/ProcessList.ts` (ProcessNotFound: show transient message, AccessDenied: show retry dialog per FR-022, CriticalProcessProtection: show warning per FR-023)
- [ ] T065 [US3] Style confirmation dialogs in `src/style.css` (modal overlay, centered dialog, button styling, focus states)
- [ ] T066 [US3] Add context menu styling in `src/style.css` (dropdown positioning, hover states)

**Checkpoint**: User Stories 1, 2, AND 3 complete. Application displays system health, process list, AND can terminate processes safely. Test independently.

---

## Phase 6: User Story 4 - Monitor Performance Trends (Priority: P2)

**Goal**: Display CPU and memory usage trend graphs over last 60 seconds

**Independent Test**: Open application, view performance graphs section showing line charts for CPU and memory. Verify charts update smoothly every 1-2 seconds with new data points, show tooltips on hover, and maintain 60-second sliding window.

### Implementation for User Story 4

- [ ] T067 [P] [US4] Install and configure Chart.js in `package.json` (add dependency)
- [ ] T068 [US4] Create PerformanceHistory class in `src/services/performance.ts` (rolling buffer for 60 seconds of data, addDataPoint method, sliding window logic)
- [ ] T069 [US4] Create PerformanceCharts UI component in `src/components/PerformanceCharts.ts` (two Chart.js line charts: CPU and memory)
- [ ] T070 [US4] Configure CPU chart in `src/components/PerformanceCharts.ts` (time on X-axis, percentage on Y-axis, smooth line, tooltips show timestamp and %)
- [ ] T071 [US4] Configure memory chart in `src/components/PerformanceCharts.ts` (time on X-axis, MB/GB on Y-axis, auto-scaling Y-axis per FR-009, tooltips)
- [ ] T072 [US4] Wire performance history to polling service in `src/services/performance.ts` (append new data points every 1-2 seconds, maintain 60-second window)
- [ ] T073 [US4] Implement chart update logic in `src/components/PerformanceCharts.ts` (smooth animation 150-300ms per FR-019, no jarring updates per FR-005)
- [ ] T074 [US4] Wire up PerformanceCharts component in `src/main.ts` (render charts, update with new data from polling service)
- [ ] T075 [US4] Style performance charts section in `src/style.css` (chart containers, responsive sizing, grid layout for two charts)
- [ ] T076 [US4] Add tooltip styling in `src/style.css` (readable font, contrast background, positioned near cursor)

**Checkpoint**: User Stories 1, 2, 3, AND 4 complete. Application displays system health, process list, process termination, AND performance trend graphs. Test independently.

---

## Phase 7: User Story 5 - View Detailed Process Information (Priority: P3)

**Goal**: Show detailed process information modal on double-click

**Independent Test**: Double-click any process in list, verify modal appears showing full path, command-line args, parent process, start time. Close modal with ESC or Close button.

### Implementation for User Story 5

- [ ] T077 [US5] Implement get_process_details Tauri command in `src-tauri/src/commands/process_ops.rs` (queries specific process by PID, returns ProcessInfo with all optional fields)
- [ ] T078 [US5] Register get_process_details command in `src-tauri/src/main.rs`
- [ ] T079 [US5] Create ProcessManager::get_details() method in `src-tauri/src/services/process_manager.rs` (queries sysinfo for specific PID, handles not found → AppError)
- [ ] T080 [P] [US5] Create Tauri command wrapper in `src/services/tauri.ts` (invokeGetProcessDetails function)
- [ ] T081 [US5] Create ProcessDetails UI component in `src/components/ProcessDetails.ts` (modal overlay, detail panel with all fields, labeled sections per FR-010)
- [ ] T082 [US5] Add tooltips for technical terms in `src/components/ProcessDetails.ts` (hover explanations per FR-017, tooltip appears after 1 second)
- [ ] T083 [US5] Wire double-click handler to ProcessList in `src/components/ProcessList.ts` (double-click row → call invokeGetProcessDetails → show modal)
- [ ] T084 [US5] Implement modal close logic in `src/components/ProcessDetails.ts` (ESC key, Close button, click outside modal)
- [ ] T085 [US5] Handle process not found error in `src/components/ProcessDetails.ts` (show transient message if process terminated between list and details request)
- [ ] T086 [US5] Style process details modal in `src/style.css` (overlay, centered panel, labeled sections, Close button, responsive)
- [ ] T087 [US5] Style tooltips in `src/style.css` (appear after 1s hover per FR-017, readable font, subtle animation)

**Checkpoint**: User Stories 1, 2, 3, 4, AND 5 complete. Application displays system health, process list, termination, trend graphs, AND detailed process info. Test independently.

---

## Phase 8: User Story 6 - Customize Interface Theme (Priority: P3)

**Goal**: Toggle between dark and light themes with persistence

**Independent Test**: Click theme toggle button, verify all UI elements switch between dark and light themes within 300ms. Close and reopen application, verify theme persists.

### Implementation for User Story 6

- [ ] T088 [US6] Implement get_preferences Tauri command in `src-tauri/src/commands/preferences.rs` (loads from %APPDATA%\rust-task-manager\config.json, returns UserPreferences, uses defaults if missing/corrupted)
- [ ] T089 [US6] Implement save_preferences Tauri command in `src-tauri/src/commands/preferences.rs` (validates, atomic write to config.json, creates directory if needed)
- [ ] T090 [US6] Register get_preferences and save_preferences commands in `src-tauri/src/main.rs`
- [ ] T091 [US6] Create PreferencesService in `src-tauri/src/services/config.rs` (load(), save(), default_preferences() methods)
- [ ] T092 [P] [US6] Create Tauri command wrappers in `src/services/tauri.ts` (invokeGetPreferences, invokeSavePreferences)
- [ ] T093 [US6] Create ThemeService in `src/services/theme.ts` (toggleTheme(), applyTheme(), persistTheme() methods)
- [ ] T094 [US6] Create ThemeToggle UI component in `src/components/ThemeToggle.ts` (button with icon, click toggles theme)
- [ ] T095 [US6] Implement theme toggle logic in `src/services/theme.ts` (toggle body class between dark-theme and light-theme, smooth CSS transition 300ms per FR-019)
- [ ] T096 [US6] Wire theme persistence in `src/services/theme.ts` (debounce 300ms, call invokeSavePreferences after theme change)
- [ ] T097 [US6] Load persisted theme on startup in `src/main.ts` (call invokeGetPreferences, apply theme before showing UI)
- [ ] T098 [US6] Wire up ThemeToggle component in `src/main.ts` (add to header, bind click handler)
- [ ] T099 [US6] Style theme toggle button in `src/style.css` (icon button, hover state, positioned in header)
- [ ] T100 [US6] Verify all CSS custom properties work in both dark and light themes in `src/style.css` (contrast ratios per FR-011, readability)

**Checkpoint**: All 6 user stories complete. Application fully functional with all P1, P2, and P3 features. Test all stories independently and together.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T101 [P] Add keyboard shortcuts in `src/main.ts` (Ctrl+R: refresh, Ctrl+F: search focus, Ctrl+T: theme toggle, Del: terminate process per FR-016)
- [ ] T102 [P] Implement responsive layout handling in `src/style.css` (window resize, works 1280x720 to 4K per FR-020)
- [ ] T103 [P] Add hardware acceleration to animations in `src/style.css` (transform and opacity only, will-change hints per FR-019)
- [ ] T104 [P] Optimize performance for 500+ processes in `src/utils/virtualization.ts` (ensure smooth scrolling per FR-013)
- [ ] T105 [P] Add error boundary handling in `src/main.ts` (catch and display all unhandled errors gracefully)
- [ ] T106 [P] Add comprehensive Rust doc comments to all public APIs in `src-tauri/src/` (/// format per constitution)
- [ ] T107 [P] Add JSDoc comments to all TypeScript exports in `src/` (per constitution)
- [ ] T108 [P] Add inline comments for complex logic in `src-tauri/src/services/` and `src/components/` (per constitution)
- [ ] T109 Run cargo fmt on all Rust code
- [ ] T110 Run cargo clippy and fix all warnings
- [ ] T111 Run ESLint on TypeScript code and fix issues
- [ ] T112 Run Prettier on TypeScript code
- [ ] T113 Validate quickstart.md instructions (prerequisite checks, dev workflow, production build)
- [ ] T114 Update README.md with feature list, screenshots placeholders, build instructions
- [ ] T115 Update CHANGELOG.md with version 0.1.0 release notes
- [ ] T116 Performance validation (startup <2s per SC-001, memory <50MB per SC-006, idle CPU <5% per SC-007)
- [ ] T117 Manual testing of all 6 user stories per acceptance scenarios in spec.md
- [ ] T118 Edge case testing (1000+ processes, rapid process changes, denied permissions, critical processes, missing data per spec.md Edge Cases)

**Checkpoint**: Application polished, documented, and ready for release. All constitution requirements met.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phases 3-8)**: All depend on Foundational phase completion
  - User Story 1 (Phase 3): Independent, can start after Phase 2
  - User Story 2 (Phase 4): Independent, can start after Phase 2
  - User Story 3 (Phase 5): Depends on User Story 2 (needs ProcessList component)
  - User Story 4 (Phase 6): Depends on User Story 1 (needs performance polling service)
  - User Story 5 (Phase 7): Depends on User Story 2 (needs ProcessList component)
  - User Story 6 (Phase 8): Independent, can start after Phase 2
- **Polish (Phase 9)**: Depends on all desired user stories being complete

### User Story Dependencies Graph

```
Phase 2 (Foundational)
    ├──> US1 (System Health) ──> US4 (Trend Graphs)
    ├──> US2 (Process List) ──> US3 (Terminate Process)
    │                        └──> US5 (Process Details)
    └──> US6 (Theme Toggle)
```

### Within Each User Story

- Models and types first (T010-T018 in Phase 2)
- Tauri commands next (backend implementation)
- Services and business logic
- Frontend components
- UI styling and polish
- Integration and wiring

### Parallel Opportunities

**Phase 1 (Setup)**: T002, T003, T004, T005, T006, T007, T008 can all run in parallel

**Phase 2 (Foundational)**: T009-T018 (all models and types) can run in parallel, T020-T022 (utilities and CSS) can run in parallel

**Phase 3 (US1)**: T024, T025, T027, T028, T029, T030, T031, T032 can run in parallel (different files)

**Phase 4 (US2)**: T043, T044 can run after T040-T042 in parallel

**Phase 5 (US3)**: T057, T058, T059, T060 can run in parallel (different components)

**Phase 6 (US4)**: T067, T068 can run in parallel

**Phase 7 (US5)**: T080, T081, T082 can run in parallel after T077-T079

**Phase 8 (US6)**: T092, T093, T094 can run in parallel after T088-T091

**Phase 9 (Polish)**: T101, T102, T103, T104, T105, T106, T107, T108 can all run in parallel

### MVP Strategy

For quickest time to value, implement in this order:

1. **Phase 1**: Setup (required)
2. **Phase 2**: Foundational (required)
3. **Phase 3**: User Story 1 - System Health (core MVP)
4. **STOP and VALIDATE**: Application launches and shows system health
5. **Phase 4**: User Story 2 - Process List (extends MVP)
6. **Phase 5**: User Story 3 - Terminate Process (completes primary use case)
7. **STOP and VALIDATE**: MVP complete - can diagnose and fix system issues
8. Continue with US4, US5, US6 as enhancements

---

## Parallel Example: User Story 1 (System Health)

```bash
# Launch all models/types in parallel (after Phase 2):
Task: "Create SystemInfo struct in src-tauri/src/models/system.rs"
Task: "Create PerformanceMetrics struct in src-tauri/src/models/performance.rs"
Task: "Create TypeScript SystemInfo interface in src/types/system.ts"
Task: "Create TypeScript PerformanceMetrics interface in src/types/performance.ts"

# Launch all UI components in parallel (after services):
Task: "Create SystemInfo UI component in src/components/SystemInfo.ts"
Task: "Create PerformanceMetrics UI component in src/components/PerformanceMetrics.ts"
Task: "Implement color coding logic in src/utils/formatters.ts"
```

---

## Parallel Example: User Story 3 (Terminate Process)

```bash
# Launch all dialog components in parallel:
Task: "Create confirmation dialog UI component in src/components/ConfirmDialog.ts"
Task: "Create critical process warning dialog in src/components/ConfirmDialog.ts"
Task: "Create UAC denial retry dialog in src/components/ConfirmDialog.ts"
Task: "Create Tauri command wrapper in src/services/tauri.ts (invokeKillProcess)"
```

---

## Implementation Strategy

### MVP First (User Stories 1-3 Only)

1. Complete Phase 1: Setup (T001-T008)
2. Complete Phase 2: Foundational (T009-T023) - **CRITICAL BLOCKER**
3. Complete Phase 3: User Story 1 (T024-T039) - System health display
4. **STOP and VALIDATE**: Test startup time <2s, metrics display correctly
5. Complete Phase 4: User Story 2 (T040-T052) - Process list with sorting/filtering
6. **STOP and VALIDATE**: Test 500+ processes, smooth scrolling, search works
7. Complete Phase 5: User Story 3 (T053-T066) - Process termination
8. **STOP and VALIDATE**: Test termination flow, UAC handling, critical process protection
9. **MVP COMPLETE**: Deploy/demo core functionality

### Incremental Delivery

Each user story adds independent value:

- **After US1**: Users can monitor system health (passive monitoring)
- **After US2**: Users can identify resource-heavy processes (diagnosis)
- **After US3**: Users can terminate problematic processes (resolution) ← **MVP COMPLETE**
- **After US4**: Users can understand performance trends (enhanced diagnosis)
- **After US5**: Users can investigate process details (power user feature)
- **After US6**: Users can customize appearance (personalization)

### Parallel Team Strategy

With multiple developers (after Phase 2 completes):

- **Developer A**: User Story 1 (System Health) → User Story 4 (Trend Graphs)
- **Developer B**: User Story 2 (Process List) → User Story 3 (Terminate) → User Story 5 (Details)
- **Developer C**: User Story 6 (Theme Toggle) → Polish tasks

Stories integrate smoothly due to clear API contracts and independent testing.

---

## Task Statistics

- **Total Tasks**: 118
- **Setup Tasks**: 8 (Phase 1)
- **Foundational Tasks**: 15 (Phase 2)
- **User Story 1 Tasks**: 16 (Phase 3)
- **User Story 2 Tasks**: 13 (Phase 4)
- **User Story 3 Tasks**: 14 (Phase 5)
- **User Story 4 Tasks**: 10 (Phase 6)
- **User Story 5 Tasks**: 11 (Phase 7)
- **User Story 6 Tasks**: 13 (Phase 8)
- **Polish Tasks**: 18 (Phase 9)
- **Parallel Opportunities**: 45+ tasks marked [P]
- **MVP Task Count**: 52 tasks (Phases 1-5: Setup + Foundational + US1 + US2 + US3)

---

## Notes

- [P] tasks = different files, can run in parallel with no dependencies
- [Story] label maps each task to specific user story for traceability
- Each user story is independently testable per acceptance scenarios in spec.md
- Manual testing strategy per constitution (no automated test framework for MVP)
- Performance targets validated in T116: startup <2s, memory <50MB, idle CPU <5%
- All constitution principles enforced: type safety (TypeScript strict mode, Rust type system), performance-first (virtualization, caching, hardware acceleration), Windows optimization (sysinfo, UAC handling), modern UI (dark/light themes, smooth animations), security (critical process protection, confirmation dialogs), documentation (Rust doc comments, JSDoc, inline comments)
- Commit after each task or logical group of related tasks
- Stop at any checkpoint to validate story independently before proceeding
- Avoid: vague tasks, same file conflicts, breaking user story independence
