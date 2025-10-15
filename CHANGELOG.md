# Changelog

All notable changes to the Rust Task Manager project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added - Phase 5: User Story 3 - Terminate Unresponsive Process (T053-T066) ✅ COMPLETE

#### Rust Backend - Process Termination
- **ProcessManager Service**: Created `src-tauri/src/services/process_manager.rs` (180+ lines)
  - `terminate_process()`: Core termination logic with comprehensive safety checks
  - Critical process validation using `is_critical_process()` from utils/windows.rs
  - Process existence verification before termination attempt
  - Graceful error handling with specific error types (ProcessNotFound, PermissionDenied, CriticalProcessProtection)
  - Post-termination verification to ensure process actually terminated
  - Helper methods: `process_exists()`, `get_process_name()` for diagnostics
- **kill_process Command**: Created `src-tauri/src/commands/process_ops.rs`
  - Async Tauri command accepting PID parameter
  - Uses managed ProcessManager state with Mutex for thread safety
  - Returns Result<(), String> with detailed error messages
  - Comprehensive unit tests for nonexistent processes and critical process protection
- **Command Registration**: Updated `src-tauri/src/main.rs`
  - Registered `kill_process` in invoke_handler
  - Added managed state: `Mutex<ProcessManager>` for safe concurrent access
  - Imported kill_process command function
- **Error Handling**: Enhanced `src-tauri/src/error.rs`
  - Added `ProcessNotFound(String)` for missing processes
  - Added `PermissionDenied(String)` for UAC/privilege issues (FR-022)
  - Added `ProcessTerminationFailed(String)` for kill signal failures
  - Existing `CriticalProcessProtection(String)` for critical processes (FR-023)

#### TypeScript Frontend - Termination Infrastructure
- **Tauri Wrapper**: Added `invokeKillProcess(pid)` to `src/services/tauri.ts`
  - Type-safe wrapper around kill_process command
  - Preserves error messages from Rust for frontend handling
  - Throws Error with original message for error classification
- **ConfirmDialog Component**: Created `src/components/ConfirmDialog.ts` (220 lines)
  - Three dialog types: 'standard', 'critical', 'uac-denied'
  - Standard confirmation (FR-006): Shows process name, PID, data loss warning
  - Critical process warning (FR-023): Strong warning with bullet points, default focus on Cancel
  - UAC retry dialog (FR-022): Informative message with Retry/Cancel buttons
  - Modal overlay with click-outside and ESC key handlers
  - Auto-focus on Cancel button for safety (especially for critical processes)
  - XSS protection with HTML escaping

#### TypeScript Frontend - ProcessList Enhancement
- **Context Menu**: Added right-click menu to `src/components/ProcessList.ts`
  - Shows "End Process" option with icon (⛔)
  - Positioned at mouse cursor location
  - Auto-closes on outside click or action
  - Attached to process table rows via data-pid attributes
- **Termination Flow**: Full integration in ProcessList component (~190 new lines)
  - `handleKillProcess()`: Initiates termination with standard confirmation dialog
  - `attemptKillProcess()`: Calls invokeKillProcess and handles all error cases
  - Error classification: Checks error message for specific error types
  - ProcessNotFound: Shows transient toast notification, auto-refreshes list
  - PermissionDenied: Shows UAC retry dialog (FR-022)
  - CriticalProcessProtection: Shows critical warning dialog (FR-023)
  - Success: Shows success toast, auto-refreshes process list
- **Toast Notifications**: Transient messages for quick feedback
  - Success (green): Process terminated successfully
  - Error (red): Generic termination failures
  - Info (blue): Process no longer exists
  - Auto-dismiss after 3 seconds with slide animations
  - Fixed position (bottom-right corner)
- **Auto-Refresh**: Added process list update callback
  - `setOnProcessListUpdate()`: Registers refresh callback
  - Wired in `src/main.ts` to trigger process polling after termination
  - Ensures UI reflects terminated process removal within 1-2 seconds

#### UI/UX Enhancements
- **Dialog Styling**: Added comprehensive CSS (~150 lines) to `src/style.css`
  - `.dialog-overlay`: Full-screen semi-transparent backdrop (z-index 9999)
  - `.dialog-content`: Centered modal with border-radius, box-shadow
  - `.dialog-critical`: Red border (2px) for critical process warnings
  - `.dialog-header`: Title section with color-coded headings
  - `.dialog-body`: Content area with proper spacing and line-height
  - `.dialog-warning`: Yellow-tinted warning box with border-left accent
  - `.dialog-critical-list`: Bullet list for critical consequences
  - `.dialog-info`: Blue-tinted info box for UAC messages
  - `.dialog-footer`: Button container with flex layout and gap
  - Button states: Hover, focus with box-shadows, transitions
  - Animations: fadeIn (overlay), slideInDown (dialog content)
- **Context Menu Styling**: Added CSS (~30 lines) to `src/style.css`
  - `.context-menu`: Dropdown with border, shadow, border-radius
  - `.context-menu-item`: Flex layout with icon and text
  - Hover state with background color change
  - fadeIn animation (0.1s) for smooth appearance
- **Toast Animations**: Added keyframe animations to `src/style.css`
  - `slideInUp`: Enters from bottom with opacity fade
  - `slideOutDown`: Exits to bottom with opacity fade
  - Used for transient success/error/info notifications

#### Functional Requirements Implemented
- ✅ **FR-006**: Confirmation dialog with data loss warning
- ✅ **FR-022**: UAC denial retry dialog with informative message
- ✅ **FR-023**: Critical process protection with strong warning, default focus on Cancel
- ✅ **Process Termination**: Full flow from right-click to process removal
- ✅ **Error Handling**: All error cases handled gracefully with appropriate UI feedback
- ✅ **Auto-Refresh**: Process list updates after successful termination

### Added - Phase 4: User Story 2 - Identify Resource-Heavy Processes (T040-T052) ✅ COMPLETE

#### Rust Backend - Process Commands
- **Process List Command**: `get_processes` command in `src-tauri/src/commands/processes.rs` - retrieves all running processes, returns Vec<ProcessInfo>
- **Command Registration**: `get_processes` registered in `src-tauri/src/main.rs` invoke_handler
- **Process Module**: Created `src-tauri/src/commands/processes.rs` with async command and tests
- **Reused Service**: Leveraged existing `get_process_list()` from Phase 2's `system_monitor.rs` service

#### TypeScript Frontend - Process Services
- **Tauri Wrapper**: `invokeGetProcesses()` in `src/services/tauri.ts` with type-safe error handling
- **Process Polling Service**: Extended `src/services/performance.ts` with ~150 lines:
  - `getProcessList()`: Fetches with 100ms cache
  - `startProcessPolling()`: 1.5-second polling interval (parallel to performance polling)
  - `stopProcessPolling()`: Cleanup function
  - `subscribeToProcessList()`: Subscriber pattern for updates
  - `fetchProcessesAndNotify()`: Graceful error handling per FR-014
- **Debounced Search**: 300ms debounce implementation per FR specifications

#### TypeScript Frontend - ProcessList Component
- **ProcessList Component**: `src/components/ProcessList.ts` (240 lines) with full state management
  - **Table Rendering**: Generates HTML table with PID, Name, CPU%, Memory, Status columns
  - **Sorting Logic**: `handleSort()` toggles asc/desc on any column, default CPU% descending
  - **Filter Logic**: `applyFilterAndSort()` case-insensitive substring match on process name
  - **Search Debounce**: `setSearchQuery()` with 300ms `window.setTimeout` per FR
  - **Performance**: Limits to 500 processes (virtualization deferred to Phase 7+)
  - **State Methods**: `showLoading()`, `showError()`, `updateProcesses()` for smooth updates

#### Application Integration
- **Main Entry Point**: `src/main.ts` updated with ProcessList integration
  - Initialized ProcessListComponent with DOM element
  - Started process polling on app startup
  - Subscribed to process updates with callback
  - Wired search input event listener with debounced filtering

#### UI Enhancements
- **Search Input**: Added to `index.html` in process-list section
  - Section-header layout with h2 and input
  - Input has id="process-search", class="search-input"
  - Placeholder "Search processes..." and aria-label for accessibility

#### Styling Additions
- **Process List CSS**: Added ~90 lines to `src/style.css`:
  - `.section-header`: Flex layout for h2 + search input
  - `.search-input`: Styled input with focus effects (border-color, box-shadow)
  - `.process-name`: Text overflow handling for long names
  - `.status-*`: Color-coded status (running=green, sleeping=muted, stopped=warning)
  - Sortable headers: Hover effects with bottom border accent
  - Monospace fonts: Applied to PID, CPU%, Memory columns
  - Sticky headers: `position: sticky` for table headers

#### Functional Requirements Implemented
- ✅ **FR-002**: Real-time updates (1.5s polling interval)
- ✅ **FR-013**: Handle >100 processes (500 process limit)
- ✅ **FR-014**: Graceful process disappearance handling
- ✅ **Sorting**: All columns sortable with asc/desc toggle
- ✅ **Filtering**: Case-insensitive search with 300ms debounce
- ✅ **Performance**: Smooth updates with subscriber pattern

### Added - Phase 3: User Story 1 - Quick System Health Check (T024-T039) ✅ COMPLETE

#### Rust Backend - Tauri Commands
- **System Info Command**: `get_system_info` command in `src-tauri/src/commands/system_info.rs` - queries sysinfo once at startup, returns SystemInfo
- **Performance Command**: `get_performance_data` command in `src-tauri/src/commands/performance.rs` - refreshes system metrics, returns PerformanceMetrics
- **Command Registration**: Both commands registered in `src-tauri/src/main.rs` via `tauri::generate_handler!` macro
- **Commands Module**: Created `src-tauri/src/commands/mod.rs` for organized command exports

#### TypeScript Frontend - Services & Utilities
- **Tauri Service**: `src/services/tauri.ts` with type-safe wrappers `invokeGetSystemInfo()` and `invokeGetPerformanceData()`
- **Performance Polling Service**: `src/services/performance.ts` with 1.5-second polling interval, 100ms cache, subscriber pattern
- **Formatters Utility**: `src/utils/formatters.ts` with:
  - `formatBytes()`: Human-readable sizes (KB, MB, GB, TB)
  - `formatPercent()`: Formatted percentages with decimals
  - `formatSpeed()`: Network/disk speeds (B/s, KB/s, MB/s, GB/s)
  - `getUsageColor()`: Color coding (green <50%, yellow 50-80%, red >80%)
  - `formatMemory()`: Combined memory display with color

#### TypeScript Frontend - UI Components
- **SystemInfo Component**: `src/components/SystemInfo.ts` vanilla TypeScript class
  - Renders OS, CPU, RAM, hostname in responsive grid
  - Loading and error states
  - XSS protection with HTML escaping
- **PerformanceMetrics Component**: `src/components/PerformanceMetrics.ts`
  - Displays CPU %, memory %, disk I/O, network speeds
  - Per-core CPU usage display (first 8 cores)
  - Color-coded metrics based on usage thresholds
  - Loading and error states per FR-021

#### Application Initialization
- **Main Entry Point**: `src/main.ts` orchestrates application flow
  - Shows loading states on startup per FR-024
  - Fetches system info once at launch
  - Starts performance polling with subscriber pattern
  - Error handling for failed metrics (displays "Error" per FR-021)
  - Async initialization with proper promise handling

#### Styling Enhancements
- **Metric Details CSS**: Added `.metric-details` and `.core-usage` styles for per-core CPU display
- **Loading Spinner**: Added spinner animation with keyframes
- **Responsive Design**: Media queries for mobile/tablet layouts
- **Color Coding**: Applied status colors throughout UI

### Validation Results - Phase 3
- ✅ **cargo check**: Compiles successfully with all Tauri commands
- ✅ **npm run lint**: All TypeScript passes strict ESLint checks
- ✅ **Type Safety**: Full type coverage with strict TypeScript mode
- ✅ **Error Handling**: Proper async/await with try-catch, no floating promises
- ✅ **Functional Requirements**: Implements FR-021 (error display), FR-024 (loading states), FR-002 (1-2 second polling)

### Added - Phase 2 Foundational (T009-T023) ✅ COMPLETE

#### Rust Backend - Data Models & Infrastructure
- **Error Handling**: AppError enum with thiserror derive, 8 error variants (SystemInfoError, PerformanceError, ProcessNotFound, AccessDenied, CriticalProcessProtection, TerminationFailed, IoError, SerializationError)
- **System Model**: SystemInfo struct with OS name/version, kernel version, CPU model/architecture/cores, total memory, hostname
- **Performance Model**: PerformanceMetrics struct with timestamp, CPU usage (overall + per-core), memory usage (used/total/percent), disk I/O speeds, network speeds
- **Process Models**: ProcessStatus enum (Running, Sleeping, Stopped, Other), ProcessInfo struct with PID, name, exe path, command args, CPU/memory usage, status, parent PID, start time, user
- **Preferences Model**: UserPreferences struct with ThemeMode enum (Dark/Light), WindowState (size/position/maximized), sort column/order
- **System Monitor Service**: Global System instance with once_cell::Lazy<Mutex<System>>, collect_performance_metrics() and get_process_list() functions
- **Windows Utilities**: Critical process protection list (csrss.exe, wininit.exe, services.exe, smss.exe, lsass.exe), is_critical_process() function, elevation check/request placeholders

#### TypeScript Frontend - Type Definitions
- **System Types**: SystemInfo interface matching Rust struct with camelCase naming
- **Performance Types**: PerformanceMetrics interface + PerformanceHistory interface for charting
- **Process Types**: ProcessStatus type ('running' | 'sleeping' | 'stopped' | 'other'), ProcessInfo interface, ProcessListState interface for UI state
- **Preferences Types**: ThemeMode type ('dark' | 'light'), WindowState and UserPreferences interfaces

#### UI Foundation
- **CSS Theme System**: Complete dark theme (default) with --bg-primary: #1a1a1a, --bg-secondary: #2d2d2d, --accent: #3b82f6, 12 color variables
- **Light Theme**: .light-theme class with inverted colors, --bg-primary: #ffffff, --text-primary: #111827
- **Layout Styles**: Global styles, flexbox header, responsive main content, section styling with borders and shadows
- **Component Styles**: System info grid (auto-fit, minmax(250px, 1fr)), performance metrics grid, process table with hover effects, sortable headers
- **Utility Styles**: Status colors (success/warning/error), custom scrollbar (8px width, styled thumb)
- **Base HTML**: Semantic structure with header, system-info section, performance section, process-list section

#### API Compatibility
- **sysinfo 0.32**: Fixed breaking changes from older API versions
  - Removed `CpuExt`, `SystemExt`, `ProcessExt` trait imports (no longer needed)
  - Updated `refresh_cpu()` → `refresh_cpu_all()`
  - Updated `global_cpu_info()` → `global_cpu_usage()`
  - Updated `refresh_processes()` to use `ProcessesToUpdate::All`
  - Fixed `OsString::to_string()` → `to_string_lossy().to_string()`

### Validation Results - Phase 2
- ✅ **cargo check**: Compiles successfully with all new modules (error, models, services, utils)
- ✅ **cargo clippy**: Passes with only expected unused code warnings (temporary until Phase 3 usage)
- ✅ **cargo fmt**: All Rust code properly formatted
- ✅ **npm run lint**: All TypeScript type files pass ESLint
- ✅ **Module Integration**: All Rust modules declared in main.rs and re-exported correctly

### Planned Features
- System information display (OS, CPU, RAM, hostname)
- Real-time performance monitoring (CPU, memory, disk I/O, network)
- Process list with sorting and filtering
- Process termination with safety checks
- Performance trend graphs (CPU and memory)
- Detailed process information view
- Dark/Light theme toggle

## [0.1.0] - 2025-10-15

### Added - Phase 1 Setup (T001-T008) ✅ VALIDATED

#### Project Foundation
- Initial project setup with Tauri 2.x framework
- Project structure verified and validated
- Minimal Tauri application with shell plugin integration

#### Documentation & Planning
- Constitution defining 7 core principles (Type Safety, Performance, Windows Optimization, Modern UI/UX, Security, TDD, Documentation)
- Feature specification with 6 user stories and 24 functional requirements
- Implementation plan with 118 tasks organized by phase
- Data model definitions for 5 core entities
- API contracts for 7 Tauri commands
- Developer quickstart guide
- Phase 1 completion report with validation results

#### Dependencies & Configuration
- **Rust Dependencies** (Cargo.toml):
  - tauri 2.x (framework core)
  - tauri-plugin-shell 2.x (shell operations)
  - sysinfo 0.32+ (Windows system monitoring)
  - serde 1.x + serde_json (serialization)
  - thiserror 1.x (error handling)
  - tokio 1.x (async runtime)
  - once_cell 1.x (lazy static initialization)
  
- **Frontend Dependencies** (package.json):
  - @tauri-apps/api 2.0.0+ (Tauri JavaScript API)
  - chart.js 4.4.0+ (performance visualizations)
  - TypeScript 5.9.3 (type safety)
  - Vite 5.0.8+ (build tool)
  - ESLint + Prettier (code quality)

#### Development Tools
- TypeScript strict mode enabled (tsconfig.json)
- ESLint configured with TypeScript support and strict rules
- Prettier configured with consistent formatting (single quotes, 2 spaces, 100 width)
- Rustfmt configured with 2021 edition rules
- CHANGELOG.md initialized (Keep a Changelog format)

#### Validation Results
- ✅ cargo check: Compiles successfully
- ✅ cargo clippy: No warnings or errors
- ✅ cargo fmt --check: All code properly formatted
- ✅ npm run lint: All TypeScript files pass linting
- ✅ npm run format: All files formatted correctly
- ✅ 137 npm packages installed successfully

### Technical Specifications
- **Rust**: 1.70+ (2021 edition), cargo 1.89.0+
- **TypeScript**: 5.9.3 with strict mode enabled
- **Framework**: Tauri 2.8.5+ (CLI 2.8.4+)
- **Target Platform**: Windows 10 (1809+) and Windows 11
- **Performance Targets**: <2s startup, <50MB RAM, <5% idle CPU
- **Build Configuration**: LTO enabled, optimized for size (opt-level="s")

[Unreleased]: https://github.com/yourusername/rust-task-manager/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/rust-task-manager/releases/tag/v0.1.0
