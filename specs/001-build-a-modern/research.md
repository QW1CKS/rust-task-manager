# Research: Modern Windows Task Manager

**Feature**: 001-build-a-modern  
**Date**: 2025-10-15  
**Status**: Complete

## Overview

This document captures technical research and decisions made during the planning phase. All major technical choices were pre-specified by the user and validated against project requirements and constitution principles.

## Technology Stack Research

### 1. Desktop Framework: Tauri 2.x

**Decision**: Use Tauri 2.x for desktop application framework

**Rationale**:
- **Performance**: Significantly smaller binaries (~3-5MB vs Electron's 100MB+)
- **Memory Efficiency**: Uses system webview instead of bundling Chromium, aligns with <50MB target
- **Security**: Rust backend provides memory safety guarantees
- **Modern**: Active development, growing ecosystem, production-ready

**Alternatives Considered**:
- **Electron**: Rejected due to large bundle size and higher memory footprint (would conflict with SC-006 target)
- **Native WPF/WinUI**: Rejected due to lack of web-based UI flexibility and steeper learning curve

**Version**: 2.x (stable, production-ready as of 2024)

**Key Features Used**:
- Tauri commands for frontend-backend communication
- Built-in window management
- System tray integration (future enhancement)
- Auto-updater capability (future enhancement)

### 2. System Monitoring: sysinfo Crate

**Decision**: Use sysinfo 0.32+ as primary system monitoring library

**Rationale**:
- **Cross-platform abstraction**: Works on Windows, macOS, Linux with platform-specific optimizations
- **Rich API**: Provides CPU, memory, disk, network, and process information
- **Performance**: Uses native OS APIs under the hood (Windows: WMI, PDH, psapi)
- **Mature**: Well-tested, actively maintained, used in production applications
- **Rust-native**: Type-safe, zero-cost abstractions

**Key APIs**:
```rust
System::new_all()              // Initialize system monitoring
system.refresh_all()           // Update all metrics
system.cpus()                  // Per-core CPU usage
system.total_memory()          // RAM information
system.processes()             // Process list
system.networks()              // Network interfaces
```

**Windows-Specific Considerations**:
- sysinfo uses `windows-sys` crate internally for Windows API access
- Provides process status (running, sleeping, stopped) via Windows thread states
- Access to process command-line arguments, executable path, parent PID
- Handles access-denied scenarios gracefully (returns partial data)

**Potential Enhancement**:
- If sysinfo proves insufficient for advanced features, can augment with direct `windows` crate for:
  - Deeper UAC integration
  - Process token manipulation
  - Service management (out of scope for MVP)

### 3. Frontend Stack: TypeScript + Vite

**Decision**: Use TypeScript with Vite build tool and vanilla component pattern

**Rationale**:
- **Type Safety**: Aligns with constitution Principle I (Type Safety & Error Handling)
- **Performance**: Vite provides instant HMR, <1s dev server startup
- **Build Size**: No framework overhead, minimal bundle size for <2s startup target
- **Modern**: ES modules, tree-shaking, optimized production builds
- **Developer Experience**: Fast iteration cycles during development

**TypeScript Configuration**:
```json
{
  "compilerOptions": {
    "strict": true,              // Constitution requirement
    "noImplicitAny": true,       // No implicit any types
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler"
  }
}
```

**Why No Framework**:
- **React/Vue**: Adds 100KB+ to bundle, overkill for this UI complexity
- **Svelte**: Considered but vanilla TS simpler for small team/solo developer
- **Component Pattern**: Organize code as classes/functions without framework lock-in

### 4. Visualization: Chart.js

**Decision**: Use Chart.js for performance trend charts

**Rationale**:
- **Mature**: Industry-standard charting library, battle-tested
- **Performance**: Hardware-accelerated canvas rendering, handles real-time updates
- **Features**: Line charts with streaming data, auto-scaling axes, tooltips
- **Bundle Size**: ~60KB minified, acceptable for <2s startup target
- **Customization**: Extensive theming options for dark/light mode

**Alternative Considered**:
- **D3.js**: Rejected as too complex for simple line charts (would add unnecessary bundle size)
- **Recharts**: React-specific, not applicable to vanilla TS approach

**Chart Configuration**:
```typescript
// CPU usage line chart
{
  type: 'line',
  data: { datasets: [{ data: performanceHistory }] },
  options: {
    animation: { duration: 0 },      // No animation for real-time
    scales: { 
      y: { min: 0, max: 100 },       // CPU percentage
      x: { type: 'time' }            // Time-series
    }
  }
}
```

### 5. Error Handling: thiserror

**Decision**: Use `thiserror` crate for custom error types

**Rationale**:
- **Type Safety**: Aligns with constitution Principle I
- **Ergonomics**: Derive macro reduces boilerplate
- **Error Context**: Can add context fields to error variants
- **Integration**: Works seamlessly with Tauri command error handling

**Error Design**:
```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("System information unavailable: {0}")]
    SystemError(String),
    
    #[error("Process {pid} not found")]
    ProcessNotFound { pid: u32 },
    
    #[error("Access denied: {operation}")]
    AccessDenied { operation: String },
    
    #[error("Failed to terminate process {pid}: {reason}")]
    TerminationFailed { pid: u32, reason: String },
}
```

## Architecture Patterns

### 6. Tauri Command Pattern

**Pattern**: Thin command layer + service layer separation

**Rationale**:
- **Testability**: Service layer can be unit tested without Tauri harness
- **Type Safety**: Commands enforce input validation at boundary
- **Error Handling**: Consistent Result<T, String> return type for IPC

**Structure**:
```rust
// Command layer (thin)
#[tauri::command]
async fn get_processes() -> Result<Vec<ProcessInfo>, String> {
    ProcessManager::get_all()
        .map_err(|e| e.to_string())
}

// Service layer (business logic)
impl ProcessManager {
    pub fn get_all() -> Result<Vec<ProcessInfo>, AppError> {
        // Implementation with proper error types
    }
}
```

### 7. Frontend State Management

**Pattern**: Simple service-based state with pub/sub for updates

**Rationale**:
- **No External Library**: Avoid Redux/MobX overhead
- **Sufficient Complexity**: Application state is simple (metrics, processes, preferences)
- **Reactive Updates**: Custom event system for component notifications

**Implementation**:
```typescript
class PerformanceService {
    private cache: PerformanceData | null = null;
    private cacheTimestamp: number = 0;
    private listeners: Set<(data: PerformanceData) => void> = new Set();
    
    async getData(): Promise<PerformanceData> {
        // 100ms cache implementation
    }
    
    subscribe(callback: (data: PerformanceData) => void): void {
        this.listeners.add(callback);
    }
}
```

### 8. Process List Virtualization

**Pattern**: Virtual scrolling with row recycling

**Rationale**:
- **Performance**: Render only visible rows (30-40) instead of 500+
- **60 FPS**: Maintains smooth scrolling even with large datasets
- **Memory**: Reduces DOM nodes from 500+ to ~50

**Implementation Strategy**:
- Calculate visible range based on scroll position
- Render visible items + 20-row buffer (top/bottom)
- Reuse DOM nodes when scrolling (update data only)
- Update on scroll with requestAnimationFrame throttling

## Performance Optimization Strategies

### 9. Startup Performance

**Target**: < 2 seconds to interactive

**Optimizations**:
1. **Lazy System Info**: Load system specs async, show loading state immediately
2. **Code Splitting**: Load Chart.js only when performance tab active (future)
3. **Minimal Bundle**: Vite tree-shaking, no unused dependencies
4. **Preload Hints**: Use Tauri preload for critical data
5. **Fast Rust Compilation**: Release builds with LTO and codegen-units=1

### 10. Runtime Performance

**Target**: < 50MB memory, < 5% CPU idle

**Optimizations**:
1. **Efficient Polling**: Only fetch data when window is focused
2. **Caching**: 100ms cache prevents duplicate IPC calls
3. **Debouncing**: Filter input debounced to 300ms
4. **Lazy Rendering**: Virtualize process list, defer non-visible updates
5. **Rust Efficiency**: Use references instead of clones, avoid allocations in hot paths

### 11. Memory Management

**Strategy**: Bounded buffers for performance history

**Implementation**:
```rust
struct PerformanceHistory {
    cpu: VecDeque<DataPoint>,     // Max 60 entries (60 seconds)
    memory: VecDeque<DataPoint>,  // Max 60 entries
}

impl PerformanceHistory {
    fn push(&mut self, data: DataPoint) {
        if self.cpu.len() >= 60 {
            self.cpu.pop_front();   // Remove oldest
        }
        self.cpu.push_back(data);
    }
}
```

## Security Considerations

### 12. Process Termination Safety

**Strategy**: Multi-layer protection

**Layers**:
1. **Critical Process List**: Hardcoded whitelist of protected Windows processes
2. **Confirmation Dialog**: User must explicitly confirm (FR-023)
3. **Privilege Check**: Detect if elevation needed before attempting
4. **Graceful Failure**: Handle access denied without crashing (FR-022)

**Critical Process List**:
```rust
const CRITICAL_PROCESSES: &[&str] = &[
    "csrss.exe",    // Client/Server Runtime Subsystem
    "wininit.exe",  // Windows Initialization
    "services.exe", // Service Control Manager
    "lsass.exe",    // Local Security Authority
    "smss.exe",     // Session Manager
];
```

### 13. Data Validation

**Strategy**: Validate at Tauri command boundary

**Validation Rules**:
- PID must be > 0 and < u32::MAX
- Process names must be valid UTF-8
- Filter queries must be sanitized (no SQL injection risk, but sanitize for display)
- Configuration file must pass schema validation before loading

## Testing Strategy

### 14. Rust Backend Testing

**Approach**: Unit tests + integration tests

**Unit Tests** (`#[cfg(test)]` modules):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_status_mapping() {
        // Test ProcessStatus enum conversions
    }
    
    #[test]
    fn test_critical_process_detection() {
        assert!(is_critical_process("csrss.exe"));
    }
}
```

**Integration Tests** (`tests/` directory):
```rust
#[test]
fn test_get_processes_command() {
    // Mock system, verify command contract
}
```

### 15. Frontend Testing

**Approach**: Manual testing for MVP, automated tests deferred to post-MVP

**Rationale**:
- Constitution requires testing for "critical user interactions"
- Manual testing sufficient for 6 user stories in MVP
- Automated frontend testing adds complexity (Playwright/Cypress setup)
- Focus resources on Rust backend testing (>80% coverage target)

**Manual Test Checklist**:
- System info displays on startup
- Performance metrics update every 1-2 seconds
- Process list sorts by each column
- Search filter works in real-time
- Process termination shows confirmation and works
- Theme toggle switches colors
- All error states display correctly (from FR-021, FR-022)

## Deployment & Build

### 16. Production Build Configuration

**Cargo Release Profile**:
```toml
[profile.release]
opt-level = "z"        # Optimize for size
lto = true             # Link-time optimization
codegen-units = 1      # Better optimization
strip = true           # Remove debug symbols
panic = "abort"        # Smaller binary
```

**Tauri Configuration** (tauri.conf.json):
```json
{
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devPath": "http://localhost:5173",
    "distDir": "../dist"
  },
  "bundle": {
    "active": true,
    "targets": ["msi", "nsis"],
    "windows": {
      "certificateThumbprint": null,  // TODO: Code signing
      "digestAlgorithm": "sha256",
      "timestampUrl": ""
    }
  }
}
```

## Open Questions & Future Considerations

### Deferred to Tasks Phase

1. **CI/CD Pipeline**: Define GitHub Actions workflow for automated testing and builds
2. **Code Signing**: Establish certificate for Windows executable signing (prevents SmartScreen warnings)
3. **Auto-Updater**: Configure Tauri updater for seamless updates (post-MVP)
4. **Installer**: Choose between MSI vs NSIS installer format

### Potential Enhancements (Post-MVP)

1. **`windows` Crate Integration**: If sysinfo limitations discovered, add direct Windows API calls
2. **GPU Monitoring**: Add GPU metrics if user hardware supports it
3. **Performance Alerts**: Background monitoring with notifications
4. **Historical Data**: Persist metrics to SQLite for long-term trends

## Conclusion

All technical decisions are finalized and align with the project constitution. No blocking issues identified. Ready to proceed to Phase 1 (data model and contracts).
