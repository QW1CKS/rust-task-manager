# Changelog

All notable changes to the Rust Task Manager project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
