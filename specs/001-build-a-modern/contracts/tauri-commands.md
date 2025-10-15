# Tauri Command Contracts

**Feature**: 001-build-a-modern  
**Date**: 2025-10-15  
**Protocol**: Tauri IPC (JSON over IPC channel)

## Overview

This document defines the contract between the Rust backend (Tauri commands) and TypeScript frontend. All commands follow Tauri's async command pattern with `Result<T, String>` return types.

## Command Naming Convention

- Snake_case for Rust function names
- camelCase for TypeScript function calls (Tauri handles conversion)
- All commands are async on both sides

## Error Handling Contract

**Rust Side**:
```rust
#[tauri::command]
async fn command_name() -> Result<ReturnType, String> {
    // Implementation returns Result
    service::do_work()
        .map_err(|e| e.to_string())  // Convert AppError to String for IPC
}
```

**TypeScript Side**:
```typescript
import { invoke } from '@tauri-apps/api/tauri';

try {
    const result = await invoke<ReturnType>('command_name');
    // Handle success
} catch (error) {
    // error is string from Rust
    console.error('Command failed:', error);
}
```

---

## Command 1: get_system_info

**Purpose**: Retrieve static system hardware and software specifications

### Rust Signature
```rust
#[tauri::command]
async fn get_system_info() -> Result<SystemInfo, String>
```

### TypeScript Usage
```typescript
import { invoke } from '@tauri-apps/api/tauri';
import { SystemInfo } from './types/system';

const systemInfo = await invoke<SystemInfo>('get_system_info');
```

### Request
- **Parameters**: None
- **Invocation**: `invoke('get_system_info')`

### Response

**Success** (HTTP 200 equivalent):
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

**Error** (Exception thrown):
```
"System information unavailable: Failed to query OS version"
```

### Implementation Notes
- Called once at application startup
- Result cached in frontend for entire session
- Uses `sysinfo::System::new_all()` and queries OS/CPU/memory
- Should complete in < 100ms

### Test Cases
1. **Happy Path**: Returns valid system info with all fields populated
2. **Partial Failure**: If some fields unavailable, returns default/empty values
3. **Complete Failure**: Returns error string if system APIs fail

---

## Command 2: get_performance_data

**Purpose**: Retrieve current real-time performance metrics

### Rust Signature
```rust
#[tauri::command]
async fn get_performance_data() -> Result<PerformanceMetrics, String>
```

### TypeScript Usage
```typescript
import { invoke } from '@tauri-apps/api/tauri';
import { PerformanceMetrics } from './types/performance';

// Called every 1-2 seconds
const metrics = await invoke<PerformanceMetrics>('get_performance_data');
```

### Request
- **Parameters**: None
- **Invocation**: `invoke('get_performance_data')`
- **Call Frequency**: Every 1-2 seconds from frontend

### Response

**Success**:
```json
{
  "timestamp": 1697389200000,
  "cpu_usage_percent": 15.7,
  "cpu_per_core": [12.5, 18.3, 14.1, 16.9, 17.2, 13.8, 15.0, 16.5],
  "memory_used": 8589934592,
  "memory_total": 17179869184,
  "memory_percent": 50.0,
  "disk_read_bps": 1048576,
  "disk_write_bps": 524288,
  "network_upload_bps": 102400,
  "network_download_bps": 2097152
}
```

**Error**:
```
"Failed to collect performance metrics: System refresh failed"
```

### Implementation Notes
- Calls `system.refresh_all()` before collecting metrics
- Timestamp is current Unix epoch in milliseconds
- CPU percentages calculated from sysinfo
- Network/disk speeds derived from byte counters (delta between calls)
- Should complete in < 50ms
- Frontend caches result for 100ms to prevent duplicate calls

### Test Cases
1. **Happy Path**: Returns valid metrics with all fields
2. **High Load**: Metrics reflect 90%+ CPU usage correctly
3. **Zero Values**: Idle system returns 0% or near-0% correctly
4. **Metric Failure**: If network unavailable, returns 0 for network speeds (FR-021)

---

## Command 3: get_processes

**Purpose**: Retrieve list of all running processes

### Rust Signature
```rust
#[tauri::command]
async fn get_processes() -> Result<Vec<ProcessInfo>, String>
```

### TypeScript Usage
```typescript
import { invoke } from '@tauri-apps/api/tauri';
import { ProcessInfo } from './types/process';

const processes = await invoke<ProcessInfo[]>('get_processes');
```

### Request
- **Parameters**: None
- **Invocation**: `invoke('get_processes')`
- **Call Frequency**: Every 1-2 seconds from frontend (when process list visible)

### Response

**Success** (array of processes):
```json
[
  {
    "pid": 1234,
    "name": "chrome.exe",
    "exe_path": "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
    "cmd_args": ["--type=renderer"],
    "cpu_percent": 12.5,
    "memory_bytes": 536870912,
    "status": "running",
    "parent_pid": 5678,
    "start_time": 1697385600,
    "user": "DESKTOP-PC\\User"
  },
  {
    "pid": 4,
    "name": "System",
    "cpu_percent": 0.1,
    "memory_bytes": 1048576,
    "status": "running",
    "parent_pid": 0,
    "start_time": 1697380000
  }
]
```

**Error**:
```
"Failed to enumerate processes: Access denied"
```

### Implementation Notes
- Calls `system.refresh_processes()` before iteration
- Iterates all processes from sysinfo
- Optional fields (`exe_path`, `cmd_args`, `user`) omitted if access denied
- Array may contain 50-500+ processes (target: handle 500+ per FR-013)
- Should complete in < 200ms even with 500+ processes
- Frontend virtualizes display if > 100 processes

### Test Cases
1. **Happy Path**: Returns valid process list with 100+ processes
2. **Access Denied**: System process has missing optional fields
3. **Process Disappears**: If process terminates during enumeration, skip it gracefully
4. **High Process Count**: 500+ processes returns successfully within 200ms

---

## Command 4: kill_process

**Purpose**: Terminate a specific process by PID

### Rust Signature
```rust
#[tauri::command]
async fn kill_process(pid: u32) -> Result<(), String>
```

### TypeScript Usage
```typescript
import { invoke } from '@tauri-apps/api/tauri';

await invoke('kill_process', { pid: 1234 });
// Returns void on success, throws string on error
```

### Request
- **Parameters**: `{ pid: number }`
- **Invocation**: `invoke('kill_process', { pid })`
- **Preconditions**: User confirmed termination via dialog (FR-006, FR-023)

### Response

**Success** (void):
```
(no return value)
```

**Errors**:

*Process Not Found*:
```
"Process 1234 not found"
```

*Access Denied*:
```
"Access denied for process 1234: Insufficient privileges"
```

*Critical Process Protection*:
```
"Critical process csrss.exe cannot be terminated safely"
```

*Termination Failed*:
```
"Failed to terminate process 1234: Process is protected"
```

### Implementation Notes
- First checks if process is in critical list (FR-023)
  - If critical, return `AppError::CriticalProcessProtection`
  - (Frontend should have already shown warning, this is backend safety check)
- Attempts to terminate using `process.kill()` from sysinfo
- May trigger Windows UAC elevation prompt if needed
- If UAC denied, returns error (FR-022 - frontend shows retry dialog)
- Should complete in < 500ms (excluding UAC interaction time)

### Security Considerations
- **Critical Process List**: `["csrss.exe", "wininit.exe", "services.exe", "lsass.exe", "smss.exe"]`
- Backend validation redundant with frontend check (defense in depth)
- Process termination is irreversible - rely on frontend confirmation

### Test Cases
1. **Happy Path**: Successfully terminates user process
2. **Process Not Found**: Returns error if PID doesn't exist
3. **Access Denied**: Returns error if insufficient privileges
4. **Critical Process**: Returns error if attempting to kill critical process
5. **UAC Elevation**: If admin rights needed, triggers UAC (manual test)

---

## Command 5: get_process_details

**Purpose**: Retrieve detailed information about a specific process

### Rust Signature
```rust
#[tauri::command]
async fn get_process_details(pid: u32) -> Result<ProcessInfo, String>
```

### TypeScript Usage
```typescript
import { invoke } from '@tauri-apps/api/tauri';
import { ProcessInfo } from './types/process';

const details = await invoke<ProcessInfo>('get_process_details', { pid: 1234 });
```

### Request
- **Parameters**: `{ pid: number }`
- **Invocation**: `invoke('get_process_details', { pid })`
- **Triggered By**: User double-clicks process in list (FR-010)

### Response

**Success**:
```json
{
  "pid": 1234,
  "name": "chrome.exe",
  "exe_path": "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
  "cmd_args": [
    "--type=renderer",
    "--enable-features=NetworkService",
    "--disable-gpu-compositing"
  ],
  "cpu_percent": 12.5,
  "memory_bytes": 536870912,
  "status": "running",
  "parent_pid": 5678,
  "start_time": 1697385600,
  "user": "DESKTOP-PC\\User"
}
```

**Error**:
```
"Process 1234 not found"
```

### Implementation Notes
- Queries specific process from sysinfo by PID
- Returns same structure as `ProcessInfo` from `get_processes`
- May have more complete optional fields if refresh is focused
- Should complete in < 50ms
- Frontend displays in modal/detail panel

### Test Cases
1. **Happy Path**: Returns detailed info for valid PID
2. **Process Not Found**: Returns error if process terminated between list view and detail request
3. **Access Denied**: Optional fields missing but returns successfully

---

## Command 6: get_preferences

**Purpose**: Load user preferences from disk

### Rust Signature
```rust
#[tauri::command]
async fn get_preferences() -> Result<UserPreferences, String>
```

### TypeScript Usage
```typescript
import { invoke } from '@tauri-apps/api/tauri';
import { UserPreferences } from './types/preferences';

const prefs = await invoke<UserPreferences>('get_preferences');
```

### Request
- **Parameters**: None
- **Invocation**: `invoke('get_preferences')`
- **Called**: Once at application startup

### Response

**Success**:
```json
{
  "theme": "dark",
  "window": {
    "width": 1200,
    "height": 800,
    "x": 100,
    "y": 100,
    "maximized": false
  },
  "sort_column": "cpu_percent",
  "sort_order": "desc"
}
```

**Error** (returns defaults):
```
(If file missing or corrupted, returns default UserPreferences)
```

### Implementation Notes
- Loads from `%APPDATA%\rust-task-manager\config.json`
- If file doesn't exist (first run), returns defaults
- If file corrupted, logs error and returns defaults
- Never throws error - always returns valid preferences
- Should complete in < 50ms

### Test Cases
1. **First Run**: File doesn't exist, returns defaults
2. **Valid Config**: Loads saved preferences correctly
3. **Corrupted File**: Returns defaults and logs warning
4. **Partial Config**: Merges saved values with defaults for missing fields

---

## Command 7: save_preferences

**Purpose**: Save user preferences to disk

### Rust Signature
```rust
#[tauri::command]
async fn save_preferences(preferences: UserPreferences) -> Result<(), String>
```

### TypeScript Usage
```typescript
import { invoke } from '@tauri-apps/api/tauri';
import { UserPreferences } from './types/preferences';

await invoke('save_preferences', { preferences });
```

### Request
- **Parameters**: `{ preferences: UserPreferences }`
- **Invocation**: `invoke('save_preferences', { preferences })`
- **Call Frequency**: Debounced (300ms after change), on app close

### Request Body Example
```json
{
  "preferences": {
    "theme": "light",
    "window": {
      "width": 1400,
      "height": 900,
      "x": 200,
      "y": 150,
      "maximized": false
    },
    "sort_column": "memory_bytes",
    "sort_order": "desc"
  }
}
```

### Response

**Success** (void):
```
(no return value)
```

**Error**:
```
"Failed to save preferences: Permission denied"
```

### Implementation Notes
- Saves to `%APPDATA%\rust-task-manager\config.json`
- Creates directory if it doesn't exist
- Atomic write (write to temp file, then rename)
- Validates preferences before saving
- Should complete in < 100ms
- Failure is non-critical - logs error but doesn't block UI

### Test Cases
1. **Happy Path**: Saves preferences successfully
2. **Directory Creation**: Creates %APPDATA% directory if missing
3. **Permission Denied**: Returns error if directory not writable
4. **Invalid Data**: Validates and rejects invalid preference values

---

## IPC Performance Targets

| Command | Target Latency | Max Payload Size | Call Frequency |
|---------|----------------|------------------|----------------|
| `get_system_info` | < 100ms | ~500 bytes | Once per session |
| `get_performance_data` | < 50ms | ~300 bytes | Every 1-2 seconds |
| `get_processes` | < 200ms | ~10-50 KB (100-500 processes) | Every 1-2 seconds |
| `kill_process` | < 500ms | ~50 bytes | User-initiated (rare) |
| `get_process_details` | < 50ms | ~500 bytes | User-initiated (occasional) |
| `get_preferences` | < 50ms | ~500 bytes | Once per session |
| `save_preferences` | < 100ms | ~500 bytes | Debounced (every 300ms+ after change) |

## Error Code Mapping

Since Tauri returns errors as strings, we use message prefixes to categorize:

| Rust Error Variant | String Prefix | Frontend Handling |
|--------------------|---------------|-------------------|
| `ProcessNotFound` | "Process {pid} not found" | Show transient message |
| `AccessDenied` | "Access denied for process" | Show FR-022 retry dialog |
| `CriticalProcessProtection` | "Critical process" | Show FR-023 strong warning |
| `TerminationFailed` | "Failed to terminate process" | Show error message |
| `SystemInfoError` | "System information unavailable" | Show FR-021 error state |
| `PerformanceError` | "Failed to collect performance metrics" | Show FR-021 error state |

## Versioning

**Current Version**: 1.0 (MVP)

**Future Compatibility**:
- Commands are additive - new commands don't break existing ones
- If command signature changes, create new command with v2 suffix (e.g., `get_processes_v2`)
- Deprecation policy: maintain old command for 1 major version

## Security Considerations

1. **Input Validation**: All PIDs validated (must be > 0 and < u32::MAX)
2. **Critical Process Protection**: Backend enforces critical process list (defense in depth)
3. **Path Sanitization**: Executable paths sanitized before display (prevent injection)
4. **Privilege Escalation**: UAC prompts only when necessary (FR-015)
5. **Data Exposure**: No sensitive user data (passwords, tokens) ever transmitted

## Testing Strategy

### Unit Tests (Rust)
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_get_system_info_success() {
        // Mock sysinfo, verify SystemInfo structure
    }
    
    #[tokio::test]
    async fn test_kill_process_critical_protection() {
        // Verify critical processes are protected
    }
}
```

### Integration Tests (Tauri Test Harness)
```rust
#[test]
fn test_command_contracts() {
    // Use Tauri test utilities to invoke commands
    // Verify serialization/deserialization
}
```

### Manual Testing
- All commands tested with real Windows system
- UAC elevation tested manually (requires actual elevation)
- Critical process protection verified with csrss.exe test

## Conclusion

All 7 Tauri commands defined with complete contracts. Ready to proceed to quickstart documentation and agent context update (Phase 1 completion).
