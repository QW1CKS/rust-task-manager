# Data Model: Modern Windows Task Manager

**Feature**: 001-build-a-modern  
**Date**: 2025-10-15  
**Status**: Complete

## Overview

This document defines the core data structures for the Windows Task Manager application. All entities are defined in both Rust (backend) and TypeScript (frontend) with serialization via serde_json.

## Core Entities

### 1. SystemInfo

**Purpose**: Represents static system hardware and software configuration

**Rust Definition** (`src-tauri/src/models/system.rs`):
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Operating system name (e.g., "Windows 11")
    pub os_name: String,
    
    /// Operating system version (e.g., "10.0.22631")
    pub os_version: String,
    
    /// Kernel version
    pub kernel_version: String,
    
    /// CPU model name (e.g., "Intel(R) Core(TM) i7-9700K")
    pub cpu_model: String,
    
    /// CPU architecture (e.g., "x86_64")
    pub cpu_architecture: String,
    
    /// Number of physical CPU cores
    pub cpu_cores: u32,
    
    /// Total RAM in bytes
    pub total_memory: u64,
    
    /// Computer hostname
    pub hostname: String,
}

impl SystemInfo {
    /// Gather system information using sysinfo crate
    pub fn gather() -> Result<Self, crate::errors::AppError> {
        // Implementation uses sysinfo::System
        unimplemented!()
    }
}
```

**TypeScript Definition** (`src/types/system.ts`):
```typescript
export interface SystemInfo {
  osName: string;
  osVersion: string;
  kernelVersion: string;
  cpuModel: string;
  cpuArchitecture: string;
  cpuCores: number;
  totalMemory: number;  // bytes
  hostname: string;
}
```

**Lifecycle**: Gathered once at application startup, cached for entire session

**Validation Rules**:
- All string fields must be non-empty
- `cpu_cores` must be > 0
- `total_memory` must be > 0

---

### 2. PerformanceMetrics

**Purpose**: Represents real-time resource utilization snapshot

**Rust Definition** (`src-tauri/src/models/performance.rs`):
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Timestamp when metrics were collected (Unix epoch milliseconds)
    pub timestamp: u64,
    
    /// Overall CPU usage percentage (0.0 - 100.0)
    pub cpu_usage_percent: f32,
    
    /// Per-core CPU usage percentages
    pub cpu_per_core: Vec<f32>,
    
    /// Used memory in bytes
    pub memory_used: u64,
    
    /// Total memory in bytes
    pub memory_total: u64,
    
    /// Memory usage percentage (0.0 - 100.0)
    pub memory_percent: f32,
    
    /// Disk read speed in bytes/second
    pub disk_read_bps: u64,
    
    /// Disk write speed in bytes/second
    pub disk_write_bps: u64,
    
    /// Network upload speed in bytes/second
    pub network_upload_bps: u64,
    
    /// Network download speed in bytes/second
    pub network_download_bps: u64,
}

impl PerformanceMetrics {
    /// Collect current performance metrics
    pub fn collect(system: &mut sysinfo::System) -> Result<Self, crate::errors::AppError> {
        // Implementation refreshes system and gathers metrics
        unimplemented!()
    }
    
    /// Calculate memory percentage from used/total
    pub fn calculate_memory_percent(used: u64, total: u64) -> f32 {
        if total == 0 { 0.0 } else { (used as f64 / total as f64 * 100.0) as f32 }
    }
}
```

**TypeScript Definition** (`src/types/performance.ts`):
```typescript
export interface PerformanceMetrics {
  timestamp: number;            // Unix epoch milliseconds
  cpuUsagePercent: number;      // 0-100
  cpuPerCore: number[];         // Array of 0-100 per core
  memoryUsed: number;           // bytes
  memoryTotal: number;          // bytes
  memoryPercent: number;        // 0-100
  diskReadBps: number;          // bytes per second
  diskWriteBps: number;         // bytes per second
  networkUploadBps: number;     // bytes per second
  networkDownloadBps: number;   // bytes per second
}

export interface PerformanceHistory {
  cpu: Array<{ timestamp: number; value: number }>;
  memory: Array<{ timestamp: number; value: number }>;
  maxPoints: number;  // 60 for 60-second window
}
```

**Lifecycle**: Collected every 1-2 seconds, maintained in 60-point rolling buffer (frontend only)

**Validation Rules**:
- `timestamp` must be recent (within last 5 seconds)
- All percentage values must be 0.0-100.0
- `memory_used` must be <= `memory_total`
- Disk and network speeds must be >= 0

---

### 3. ProcessInfo

**Purpose**: Represents a running process or service

**Rust Definition** (`src-tauri/src/models/process.rs`):
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProcessStatus {
    Running,
    Sleeping,
    Stopped,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Process ID (unique identifier)
    pub pid: u32,
    
    /// Process name (executable name)
    pub name: String,
    
    /// Full executable path (may be empty if inaccessible)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exe_path: Option<String>,
    
    /// Command-line arguments (may be empty if inaccessible)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd_args: Option<Vec<String>>,
    
    /// CPU usage percentage (0.0 - 100.0)
    pub cpu_percent: f32,
    
    /// Memory usage in bytes
    pub memory_bytes: u64,
    
    /// Current process status
    pub status: ProcessStatus,
    
    /// Parent process ID (0 if no parent or inaccessible)
    pub parent_pid: u32,
    
    /// Process start time (Unix epoch seconds)
    pub start_time: u64,
    
    /// User account running the process (may be empty if inaccessible)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl ProcessInfo {
    /// Convert from sysinfo::Process
    pub fn from_sysinfo(pid: sysinfo::Pid, process: &sysinfo::Process) -> Self {
        // Implementation maps sysinfo data to ProcessInfo
        unimplemented!()
    }
    
    /// Check if this is a critical Windows system process
    pub fn is_critical(&self) -> bool {
        const CRITICAL: &[&str] = &[
            "csrss.exe", "wininit.exe", "services.exe", 
            "lsass.exe", "smss.exe"
        ];
        CRITICAL.contains(&self.name.to_lowercase().as_str())
    }
}
```

**TypeScript Definition** (`src/types/process.ts`):
```typescript
export type ProcessStatus = 'running' | 'sleeping' | 'stopped' | 'other';

export interface ProcessInfo {
  pid: number;
  name: string;
  exePath?: string;              // Optional: access may be denied
  cmdArgs?: string[];            // Optional: access may be denied
  cpuPercent: number;            // 0-100
  memoryBytes: number;
  status: ProcessStatus;
  parentPid: number;             // 0 if no parent
  startTime: number;             // Unix epoch seconds
  user?: string;                 // Optional: access may be denied
}

export interface ProcessListState {
  processes: ProcessInfo[];
  sortColumn: keyof ProcessInfo;
  sortOrder: 'asc' | 'desc';
  filterQuery: string;
}
```

**Lifecycle**: Dynamic - collected every refresh cycle, can appear/disappear between polls

**Validation Rules**:
- `pid` must be > 0 and within valid range for target platform
- `name` must be non-empty
- `cpu_percent` must be 0.0-100.0
- `memory_bytes` must be >= 0
- `start_time` must be <= current time
- Optional fields (`exe_path`, `cmd_args`, `user`) may be None if access denied

**State Transitions**:
```
[New Process] → Running
Running ↔ Sleeping
Running → Stopped → [Terminated]
Running → Other (rare edge case)
```

---

### 4. UserPreferences

**Purpose**: Persisted user settings and application state

**Rust Definition** (`src-tauri/src/models/preferences.rs`):
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Dark,
    Light,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Dark  // Constitution: dark mode default
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            x: 0,    // Centered by Tauri
            y: 0,
            maximized: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserPreferences {
    /// Theme mode (dark/light)
    pub theme: ThemeMode,
    
    /// Window position and size
    pub window: WindowState,
    
    /// Last selected sort column for process list
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_column: Option<String>,
    
    /// Last selected sort order
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<String>,
}

impl UserPreferences {
    /// Load preferences from config file
    pub fn load() -> Result<Self, crate::errors::AppError> {
        // Load from %APPDATA%/rust-task-manager/config.json
        unimplemented!()
    }
    
    /// Save preferences to config file
    pub fn save(&self) -> Result<(), crate::errors::AppError> {
        // Save to %APPDATA%/rust-task-manager/config.json
        unimplemented!()
    }
}
```

**TypeScript Definition** (`src/types/preferences.ts`):
```typescript
export type ThemeMode = 'dark' | 'light';

export interface WindowState {
  width: number;
  height: number;
  x: number;
  y: number;
  maximized: boolean;
}

export interface UserPreferences {
  theme: ThemeMode;
  window: WindowState;
  sortColumn?: string;
  sortOrder?: string;
}
```

**Storage Location**: `%APPDATA%\rust-task-manager\config.json` on Windows

**Lifecycle**: Loaded on startup, saved on changes (debounced), saved on graceful shutdown

**Validation Rules**:
- `theme` must be "dark" or "light"
- Window dimensions must be >= 800x600 (minimum size)
- `sort_column` must match valid ProcessInfo field names if present

---

### 5. AppError (Error Handling)

**Purpose**: Type-safe error variants for application errors

**Rust Definition** (`src-tauri/src/errors/app_error.rs`):
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("System information unavailable: {0}")]
    SystemInfoError(String),
    
    #[error("Failed to collect performance metrics: {0}")]
    PerformanceError(String),
    
    #[error("Process {pid} not found")]
    ProcessNotFound { pid: u32 },
    
    #[error("Access denied for process {pid}: {reason}")]
    AccessDenied { pid: u32, reason: String },
    
    #[error("Failed to terminate process {pid}: {reason}")]
    TerminationFailed { pid: u32, reason: String },
    
    #[error("Critical process {name} cannot be terminated safely")]
    CriticalProcessProtection { name: String },
    
    #[error("Failed to load preferences: {0}")]
    PreferencesLoadError(String),
    
    #[error("Failed to save preferences: {0}")]
    PreferencesSaveError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

impl AppError {
    /// Convert to user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Self::ProcessNotFound { pid } => 
                format!("Process {} no longer exists", pid),
            Self::AccessDenied { pid, .. } => 
                format!("Cannot access process {} (permission denied)", pid),
            Self::CriticalProcessProtection { name } => 
                format!("{} is a critical system process", name),
            _ => self.to_string(),
        }
    }
}
```

**TypeScript Definition** (error responses are strings from Tauri):
```typescript
export interface TauriError {
  message: string;
  type?: string;
}

export function handleTauriError(error: unknown): string {
  if (typeof error === 'string') return error;
  if (error instanceof Error) return error.message;
  return 'An unexpected error occurred';
}
```

---

## Data Flow

### Startup Sequence
```
1. Application Launch
   ↓
2. Load UserPreferences from disk
   ↓
3. Gather SystemInfo (cached)
   ↓
4. Display UI with loading state (FR-024)
   ↓
5. Collect initial PerformanceMetrics
   ↓
6. Collect initial ProcessInfo list
   ↓
7. UI becomes interactive (< 2 seconds)
```

### Runtime Polling
```
Frontend Timer (every 1-2 seconds)
   ↓
Get cached PerformanceMetrics (100ms cache)
   ↓
Update performance charts (CPU, memory trends)
   ↓
Get ProcessInfo list
   ↓
Update process table (preserve scroll position)
```

### Process Termination Flow
```
User clicks "End Process"
   ↓
Check if critical process (is_critical())
   ↓
Show appropriate confirmation dialog (FR-023 or FR-006)
   ↓
User confirms
   ↓
Tauri command: kill_process(pid)
   ↓
Backend checks privileges, may trigger UAC (FR-015)
   ↓
If denied: Show retry dialog (FR-022)
   ↓
If success: Process removed from list
```

### Preference Persistence Flow
```
User changes theme/window size
   ↓
Update UserPreferences in memory
   ↓
Debounce save (300ms)
   ↓
Save to config.json
   ↓
On error: Log but don't block UI
```

## Serialization Examples

### JSON Serialization (Rust → TypeScript)

**SystemInfo Example**:
```json
{
  "os_name": "Windows 11",
  "os_version": "10.0.22631",
  "kernel_version": "22631.1.amd64fre",
  "cpu_model": "Intel(R) Core(TM) i7-9700K CPU @ 3.60GHz",
  "cpu_architecture": "x86_64",
  "cpu_cores": 8,
  "total_memory": 17179869184,
  "hostname": "DESKTOP-PC"
}
```

**ProcessInfo Example**:
```json
{
  "pid": 1234,
  "name": "chrome.exe",
  "exe_path": "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
  "cmd_args": ["--type=renderer", "--enable-features=..."],
  "cpu_percent": 12.5,
  "memory_bytes": 536870912,
  "status": "running",
  "parent_pid": 5678,
  "start_time": 1697385600,
  "user": "DESKTOP-PC\\User"
}
```

**ProcessInfo with Access Denied**:
```json
{
  "pid": 4,
  "name": "System",
  "cpu_percent": 0.1,
  "memory_bytes": 1048576,
  "status": "running",
  "parent_pid": 0,
  "start_time": 1697380000
}
```
*(Note: `exe_path`, `cmd_args`, `user` omitted due to access restrictions)*

## Validation & Constraints

### Cross-Entity Constraints

1. **Memory Consistency**: `PerformanceMetrics.memory_total` MUST match `SystemInfo.total_memory`
2. **Process Parent**: If `ProcessInfo.parent_pid` != 0, parent process should exist in process list (but may not due to race conditions)
3. **Timestamp Ordering**: `PerformanceMetrics.timestamp` should always increase (monotonic)
4. **CPU Core Count**: Length of `PerformanceMetrics.cpu_per_core` MUST equal `SystemInfo.cpu_cores`

### Field Constraints

**Memory Values**:
- All memory values in bytes (u64)
- Display conversion: bytes → KB → MB → GB (1024-based)

**Percentages**:
- Always 0.0-100.0 range
- Precision: 1 decimal place in UI (e.g., "12.5%")

**Timestamps**:
- Unix epoch (seconds for process start time, milliseconds for metrics timestamp)
- Always UTC internally, display in local time

**PIDs**:
- Positive integers (u32)
- PID 0 reserved for "no parent"
- PID may be reused by OS after process terminates

## Future Extensions

### Potential Additions (Post-MVP)

1. **GPUMetrics**: GPU utilization, VRAM usage (requires additional crate)
2. **NetworkConnection**: Per-process network connections (requires Windows API)
3. **DiskUsage**: Per-drive storage breakdown (out of scope per spec)
4. **ProcessHistory**: Historical data for individual processes (performance impact)

### Schema Versioning

If data model changes in future:
1. Add `version` field to UserPreferences
2. Implement migration logic in `load()` method
3. Maintain backward compatibility for 1 major version

## Conclusion

All core entities defined with full type safety in both Rust and TypeScript. Serialization via serde_json provides seamless IPC between backend and frontend. Ready to proceed to contract definition (Phase 1 continued).
