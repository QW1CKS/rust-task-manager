# 🚀 Quickstart Guide

**Project**: Rust Task Manager  
**Last Updated**: 2025-10-15  
**Target Platform**: Windows 10 (1809+) and Windows 11 (x64)

## Overview

This is a modern Windows task manager built with **Tauri 2.x** (Rust backend) and **TypeScript** (frontend). It provides real-time system monitoring, process management, and performance visualization.

**Key Technologies**:
- **Backend**: Rust 1.70+ with `sysinfo` crate for system metrics
- **Frontend**: TypeScript (strict mode) + Vite + Chart.js
- **Framework**: Tauri 2.x for native Windows desktop app
- **Performance**: <2s startup, <50MB RAM, <5% idle CPU

---

## Prerequisites

### Required Software

1. **Rust 1.70 or later**
   ```powershell
   # Install via rustup (if not installed)
   winget install Rustlang.Rustup
   
   # Verify installation
   rustc --version  # Should show 1.70.0 or later
   cargo --version
   ```

2. **Node.js 18+ and npm**
   ```powershell
   # Install Node.js
   winget install OpenJS.NodeJS.LTS
   
   # Verify installation
   node --version   # Should show v18.0.0 or later
   npm --version
   ```

3. **Visual Studio C++ Build Tools** (for Rust compilation)
   ```powershell
   # Install via Visual Studio Installer
   winget install Microsoft.VisualStudio.2022.BuildTools
   
   # Or use the minimal installer
   # Select "Desktop development with C++" workload
   ```

4. **WebView2** (usually pre-installed on Windows 11)
   ```powershell
   # Check if installed (should return version)
   Get-ItemProperty "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" -Name "pv"
   
   # If not installed, download from:
   # https://developer.microsoft.com/en-us/microsoft-edge/webview2/
   ```

### Recommended Tools

- **VS Code** with extensions:
  - rust-analyzer (Rust language support)
  - Tauri (Tauri development tools)
  - ESLint (TypeScript linting)
  - Prettier (code formatting)

---

## Getting Started

### 1. Clone and Install Dependencies

```powershell
# Clone repository (if not already)
git clone https://github.com/yourusername/rust-task-manager.git
cd rust-task-manager\rust-task-manager

# Install frontend dependencies
npm install

# Verify Tauri CLI
npm run tauri --version  # Should show Tauri CLI version 2.x
```

### 2. Development Mode (Hot Reload)

```powershell
# Start development server (frontend + backend)
npm run tauri:dev

# This will:
# 1. Start Vite dev server (frontend) on http://localhost:5173
# 2. Compile Rust backend
# 3. Launch Tauri window with hot reload
```

**What to Expect**:
- First compile takes 2-5 minutes (builds Rust dependencies)
- Subsequent compiles are faster (~10-30 seconds)
- Frontend changes hot reload instantly
- Rust changes require recompilation (automatic)

**Troubleshooting**:
- If compile fails, ensure Visual Studio C++ Build Tools installed
- If "WebView2 not found", install from link above
- If port 5173 busy, Vite will use next available port

### 3. Production Build

```powershell
# Build optimized executable
npm run tauri:build

# Output location:
# src-tauri\target\release\rust-task-manager.exe (unpackaged)
# src-tauri\target\release\bundle\nsis\*.exe (installer)
```

**Build Artifacts**:
- **EXE**: Standalone executable (~3-5 MB)
- **NSIS Installer**: Windows installer with uninstaller
- **MSI**: Windows Installer package (optional, configure in `tauri.conf.json`)

**Performance Validation**:
- Startup time: <2 seconds (test with fresh launch)
- Memory footprint: <50 MB (check Task Manager)
- Idle CPU: <5% (monitor over 1 minute)

---

## Project Structure

```
rust-task-manager/
├── src/                      # Frontend (TypeScript)
│   ├── main.ts               # Entry point, Tauri command calls
│   ├── style.css             # Global styles
│   ├── components/           # UI components (future)
│   ├── services/             # Frontend services (future)
│   └── types/                # TypeScript type definitions
│       ├── system.ts         # SystemInfo interface
│       ├── performance.ts    # PerformanceMetrics interface
│       ├── process.ts        # ProcessInfo, ProcessStatus
│       └── preferences.ts    # UserPreferences interface
│
├── src-tauri/                # Backend (Rust)
│   ├── src/
│   │   ├── main.rs           # Entry point, Tauri command registration
│   │   ├── commands/         # Tauri commands (IPC layer)
│   │   │   ├── mod.rs
│   │   │   ├── system.rs     # get_system_info
│   │   │   ├── performance.rs # get_performance_data
│   │   │   ├── processes.rs  # get_processes, kill_process, get_process_details
│   │   │   └── preferences.rs # get_preferences, save_preferences
│   │   ├── services/         # Business logic
│   │   │   ├── mod.rs
│   │   │   ├── system_service.rs
│   │   │   ├── process_service.rs
│   │   │   └── preferences_service.rs
│   │   ├── models/           # Data structures
│   │   │   ├── mod.rs
│   │   │   ├── system_info.rs
│   │   │   ├── performance_metrics.rs
│   │   │   ├── process_info.rs
│   │   │   └── user_preferences.rs
│   │   └── error.rs          # AppError type (using thiserror)
│   ├── Cargo.toml            # Rust dependencies
│   └── tauri.conf.json       # Tauri configuration
│
├── specs/                    # Specification documents
│   └── 001-build-a-modern/
│       ├── spec.md           # Feature requirements
│       ├── plan.md           # Implementation plan
│       ├── data-model.md     # Data structure definitions
│       ├── research.md       # Technical research
│       └── contracts/        # API contracts
│           └── tauri-commands.md
│
├── package.json              # NPM scripts and frontend deps
├── tsconfig.json             # TypeScript configuration
├── vite.config.ts            # Vite bundler configuration
└── SPECIFICATION.md          # High-level project specification
```

---

## Development Workflow

### Daily Development

1. **Start Development Server**:
   ```powershell
   npm run tauri:dev
   ```

2. **Make Changes**:
   - **Frontend** (TypeScript/HTML/CSS): Save file → instant hot reload
   - **Backend** (Rust): Save file → automatic recompile (10-30s)

3. **Test Locally**:
   - Application window updates automatically
   - Use browser DevTools: Right-click → Inspect Element
   - Check Rust logs in terminal where `tauri:dev` is running

4. **Commit Changes**:
   ```powershell
   git add .
   git commit -m "feat: add CPU usage graph"
   git push
   ```

### Common Commands

```powershell
# Development
npm run tauri:dev       # Start dev server with hot reload
npm run dev             # Start Vite only (for frontend-only testing)

# Building
npm run tauri:build     # Production build with installer
npm run build           # Frontend build only (outputs to dist/)

# Testing
cargo test              # Run Rust unit tests
npm test                # (Future) Frontend tests

# Code Quality
cargo fmt               # Format Rust code
cargo clippy            # Lint Rust code
npm run lint            # (Future) Lint TypeScript

# Debugging
cargo run               # Run backend without Tauri (for debugging)
npm run tauri:build --debug  # Debug build with symbols
```

### Opening DevTools in Tauri

**Development Mode**:
- Right-click in window → "Inspect Element"
- Or press `F12`
- Or press `Ctrl+Shift+I`

**Production Mode**:
- DevTools disabled by default in `tauri.conf.json`
- To enable: set `devtools: true` in config

---

## Testing Strategy (MVP)

### Backend Testing (Rust)

**Unit Tests**:
```powershell
# Run all tests
cargo test

# Run specific module
cargo test commands::system

# Run with output
cargo test -- --nocapture
```

**Example Test**:
```rust
// src-tauri/src/commands/processes.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_get_processes_returns_data() {
        let result = get_processes().await;
        assert!(result.is_ok());
        let processes = result.unwrap();
        assert!(processes.len() > 0);
    }
}
```

### Frontend Testing (Manual for MVP)

**Manual Test Checklist**:
- [ ] Application launches in <2 seconds
- [ ] System info displayed correctly (OS, CPU, RAM)
- [ ] CPU graph updates every 1-2 seconds
- [ ] Memory graph updates every 1-2 seconds
- [ ] Process list displays 100+ processes
- [ ] Process list is searchable/filterable
- [ ] Process list is sortable by CPU/memory
- [ ] Double-click process shows details modal
- [ ] Terminate process shows confirmation dialog
- [ ] Terminate critical process shows strong warning
- [ ] UAC elevation prompt appears when needed
- [ ] Theme toggle works (light/dark)
- [ ] Window size/position persisted on restart

**Performance Validation**:
```powershell
# Check startup time (run multiple times)
Measure-Command { Start-Process "src-tauri\target\release\rust-task-manager.exe" -Wait }

# Monitor memory usage (run app, then check)
Get-Process rust-task-manager | Select-Object PM

# Check CPU usage (sample over 60 seconds)
Get-Counter "\Process(rust-task-manager)\% Processor Time" -SampleInterval 1 -MaxSamples 60
```

---

## Debugging Tips

### Rust Backend Debugging

1. **Print Debugging**:
   ```rust
   println!("Debug: CPU usage = {}%", cpu_usage);
   eprintln!("Error: {}", error);  // Prints to stderr
   ```

2. **Structured Logging** (recommended):
   ```rust
   // Add to Cargo.toml: log = "0.4", env_logger = "0.10"
   log::info!("System info collected: {:?}", system_info);
   log::error!("Failed to get processes: {}", error);
   ```

3. **VS Code Debugging**:
   - Install "CodeLLDB" extension
   - Add breakpoints in Rust code
   - Run "Debug" from VS Code Run menu

### Frontend Debugging

1. **Console Logging**:
   ```typescript
   console.log('CPU usage:', metrics.cpu_usage_percent);
   console.error('Failed to invoke command:', error);
   ```

2. **Chrome DevTools**:
   - Use Network tab to see IPC calls (experimental)
   - Use Console tab for errors
   - Use Performance tab to profile rendering

3. **TypeScript Type Checking**:
   ```powershell
   npx tsc --noEmit  # Check types without building
   ```

### Common Issues

**Issue**: "Failed to run cargo build"  
**Solution**: Ensure Visual Studio C++ Build Tools installed, restart terminal

**Issue**: "WebView2 not found"  
**Solution**: Download WebView2 runtime from Microsoft

**Issue**: "Port 5173 already in use"  
**Solution**: Vite will auto-increment port, check console for actual URL

**Issue**: "Access denied when killing process"  
**Solution**: Expected for system processes, UAC elevation will prompt

**Issue**: "App crashes on startup"  
**Solution**: Check Rust panic messages in terminal, ensure `sysinfo` can query system

---

## Performance Optimization Tips

### Frontend

1. **Virtualize Process List**:
   - Only render visible rows (~20-30 at a time)
   - Use `IntersectionObserver` or library like `react-window` (if using React)

2. **Debounce Expensive Operations**:
   - Search/filter: 300ms debounce
   - Window resize: 100ms debounce

3. **Cache IPC Results**:
   - `get_system_info`: Cache entire session (static data)
   - `get_performance_data`: Cache 100ms window

4. **Optimize Chart.js**:
   - Limit data points to 60 (1 minute of history)
   - Use `decimation` plugin for high-frequency data

### Backend

1. **Minimize System Refreshes**:
   ```rust
   // Only refresh what you need
   system.refresh_cpu();  // Instead of refresh_all()
   ```

2. **Batch Process Queries**:
   - Collect all process data in single `get_processes` call
   - Avoid multiple calls per render

3. **Use Async Efficiently**:
   - Heavy operations in `tokio::task::spawn_blocking`
   - Don't block Tauri's main thread

---

## Next Steps

### Phase 1: Core Implementation (Current)
- [x] Project setup complete
- [x] Specification finalized
- [x] Data model defined
- [x] API contracts defined
- [ ] Implement Tauri commands (7 commands)
- [ ] Implement frontend UI (system info + process list)
- [ ] Add Chart.js graphs (CPU + memory)
- [ ] Manual testing and validation

### Phase 2: Polish (Future)
- [ ] Add automated frontend tests (Playwright)
- [ ] Implement process details modal (FR-010)
- [ ] Add search/filter functionality (FR-004)
- [ ] Persist user preferences (theme, window size)
- [ ] Performance profiling and optimization

### Phase 3: Advanced Features (Future)
- [ ] Startup programs management
- [ ] Service control panel
- [ ] Resource usage history (longer time window)
- [ ] Export metrics to CSV/JSON

---

## Resources

### Documentation
- **Tauri Documentation**: https://tauri.app/v2/
- **sysinfo Crate**: https://docs.rs/sysinfo/latest/sysinfo/
- **Chart.js**: https://www.chartjs.org/docs/latest/
- **TypeScript**: https://www.typescriptlang.org/docs/

### Project Specifications
- **Feature Spec**: `specs/001-build-a-modern/spec.md`
- **Implementation Plan**: `specs/001-build-a-modern/plan.md`
- **Data Model**: `specs/001-build-a-modern/data-model.md`
- **API Contracts**: `specs/001-build-a-modern/contracts/tauri-commands.md`

### Community
- **Tauri Discord**: https://discord.gg/tauri
- **Rust Community**: https://www.rust-lang.org/community

---

## Contributing

### Before Submitting PR

1. **Run Quality Checks**:
   ```powershell
   cargo fmt --check       # Verify Rust formatting
   cargo clippy            # Check Rust lints
   cargo test              # Run all tests
   npm run build           # Verify frontend builds
   ```

2. **Manual Testing**:
   - Test all modified features in `tauri:dev` mode
   - Test production build (`tauri:build`)
   - Verify performance targets (startup, memory, CPU)

3. **Update Documentation**:
   - Update `CHANGELOG.md` with changes
   - Update this `quickstart.md` if workflow changes
   - Update `README.md` if public-facing changes

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

**Example**:
```
feat(processes): add process termination command

- Implement kill_process Tauri command
- Add UAC elevation handling
- Add critical process protection check

Fixes #42
```

---

## Troubleshooting

### Build Errors

**Error**: `linker 'link.exe' not found`  
**Solution**: Install Visual Studio C++ Build Tools, restart terminal

**Error**: `error: failed to run custom build command for 'tauri-build'`  
**Solution**: Update Rust: `rustup update`, clean build: `cargo clean`

**Error**: `Could not find WebView2 runtime`  
**Solution**: Install WebView2 from Microsoft download page

### Runtime Errors

**Error**: `Os { code: 5, kind: PermissionDenied, message: "Access is denied." }`  
**Solution**: Expected for protected processes, ensure UAC prompt appears

**Error**: `Failed to invoke command: get_processes`  
**Solution**: Check Rust logs in terminal, likely sysinfo initialization issue

**Error**: Application crashes on startup (no window)  
**Solution**: Check Rust panic in terminal, ensure no conflicting software (antivirus)

### Performance Issues

**Issue**: Slow startup (>5 seconds)  
**Solution**: Check disk I/O, disable debug builds, verify SSD performance

**Issue**: High CPU usage (>20% idle)  
**Solution**: Increase polling interval (2s instead of 1s), optimize Chart.js rendering

**Issue**: High memory usage (>100 MB)  
**Solution**: Limit process list size, reduce Chart.js history window

---

## Contact

For questions or issues:
- Open GitHub issue: [Repository Issues](https://github.com/yourusername/rust-task-manager/issues)
- Check specification documents in `specs/` directory
- Review Tauri documentation for framework-specific questions

---

**Happy Coding! 🦀**
