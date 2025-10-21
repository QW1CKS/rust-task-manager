# Rust Task Manager Constitution
**A Native High-Performance Windows System Monitor**

## Core Principles

### I. Native-First Architecture (NON-NEGOTIABLE)
**Pure Rust with zero compromises on native Windows integration**

- **Direct Windows API**: All system integration through `windows-rs` crate exclusively
- **No abstraction layers**: Direct kernel-level access for real-time system metrics
- **Hardware acceleration**: DirectX 12/Direct2D rendering pipeline for UI
- **Native feel**: Windows 11 Fluent Design System with Mica/Acrylic materials
- **Zero web tech**: No Electron, no WebView2, no JavaScript - pure native code only

**Forbidden**: Cross-platform UI abstractions, web-based rendering, generic system APIs that sacrifice Windows-specific optimizations.

### II. Extreme Performance Targets (MEASURED & ENFORCED)
**Every millisecond and megabyte counts**

#### Startup Performance
- **Cold start**: <500ms from launch to fully interactive UI
- **Warm start**: <150ms when already in memory
- **Benchmark target**: 80% faster than Windows Task Manager (CTM: ~2.5s)

#### Resource Footprint
- **Idle memory**: <15MB working set
- **Active monitoring**: <25MB working set
- **CPU usage**: <2% during continuous monitoring at 1Hz refresh
- **Disk I/O**: Zero background disk activity when idle

#### Responsiveness
- **UI frame time**: <8ms (120 FPS capable)
- **Interaction latency**: <16ms for all user inputs
- **Data refresh**: Sub-millisecond metric collection cycles
- **Zero frame drops**: During process enumeration or metric updates

**Enforcement**: Performance regression tests mandatory; CI fails on budget violations; All PRs must include flamegraph analysis for hot paths.

### III. Zero-Allocation Hot Paths (CRITICAL OPTIMIZATION)
**Memory allocations are the enemy of performance**

- **Pre-allocated buffers**: All monitoring data structures sized at startup
- **Arena allocators**: Custom allocators for short-lived objects
- **Object pooling**: Reuse structures for repeated operations
- **Stack allocation**: Prefer stack over heap wherever possible
- **No collection resizing**: Pre-sized vectors and hashmaps in hot loops

**Hot path definition**: Any code executed more than 10 times per second.

**Measurement**: `#[no_alloc]` attribute enforcement via custom lints; Allocation tracking in benchmarks; Zero allocations in metric collection loops.

### IV. Strategic Unsafe Rust
**Performance-critical only, with rigorous safety contracts**

#### When Unsafe Is Permitted
- Direct Windows API FFI calls
- Zero-copy data structure manipulation
- SIMD operations for data processing
- Custom memory allocator implementations
- Lock-free concurrent data structures

#### Safety Requirements (NON-NEGOTIABLE)
- **Documentation**: Every `unsafe` block must have safety contract comment
- **Encapsulation**: Unsafe code must be wrapped in safe abstractions
- **Testing**: Miri validation required for all unsafe code paths
- **Review**: Two-person review mandatory for new unsafe code
- **Justification**: Performance benefit must be >20% to justify unsafe

**Example Safety Contract**:
```rust
// SAFETY: The Windows API guarantees `process_handle` remains valid
// for the duration of this call because we hold a PROCESS_QUERY_LIMITED_INFORMATION
// handle obtained from OpenProcess with ERROR_SUCCESS return code.
unsafe { GetProcessMemoryInfo(process_handle, &mut pmc, size) }
```

### V. Windows Integration Excellence
**Deep OS integration, not shallow compatibility**

#### Modern Windows APIs (Windows 10 1809+)
- **WinRT APIs**: For modern Windows 11 features (notification center, share, etc.)
- **Direct2D/DirectWrite**: Hardware-accelerated text and graphics
- **DXGI**: Monitor enumeration and per-monitor DPI v2 awareness
- **Windows.UI.Composition**: Fluent animations and effects
- **NT Kernel APIs**: Direct `NtQuerySystemInformation` for performance

#### Visual Integration
- **Mica material**: Translucent title bar with desktop wallpaper tint
- **Acrylic backgrounds**: Blur effects on supporting hardware
- **Fluent icons**: Segoe Fluent Icons font
- **Dark/light themes**: Automatic system theme detection and switching
- **Accent color**: System accent color integration

#### Behavioral Integration
- **High-DPI**: Per-monitor v2 DPI scaling with crisp rendering at all scales
- **Touch/pen**: First-class touch and Surface Pen support
- **Keyboard nav**: Full keyboard navigation with Windows standards
- **Screen readers**: Narrator and UI Automation support
- **Snap layouts**: Windows 11 snap assist integration

### VI. Security Hardening
**Trust boundaries respected at every layer**

#### Privilege Management
- **Least privilege default**: Launch without admin, gracefully degrade
- **Selective elevation**: Per-operation elevation requests (not app-wide)
- **Security descriptors**: Proper token manipulation for elevated operations
- **Impersonation**: Support running as different user context when needed

#### Memory Safety
- **No unsafe memory bugs**: Even in unsafe blocks, maintain Rust guarantees
- **Bounds checking**: Array access validation in critical paths
- **Integer overflow**: Checked arithmetic in security-sensitive code
- **Side-channel resistance**: Constant-time operations for sensitive data

#### Attack Surface Reduction
- **Minimal dependencies**: Audit all crates, prefer smaller dependency trees
- **No dynamic code**: No runtime code generation or plugin loading from disk
- **Input validation**: All external inputs (command line, files, registry) validated
- **Secure defaults**: Safe configuration out-of-box, require opt-in for risky features

#### Audit Trail
- **Security events**: Log all privilege escalations and sensitive operations
- **Telemetry**: Opt-in diagnostic data collection with full transparency
- **Crash dumps**: Sanitized crash reports with PII removal

### VII. Quality & Reliability (NON-NEGOTIABLE)
**Rock-solid stability under all conditions**

#### Testing Requirements
- **Code coverage**: >90% line coverage, >85% branch coverage
- **Unit tests**: Every public function must have tests
- **Integration tests**: All Windows API interactions tested with mocks
- **Property tests**: Fuzzing for parsers and data structures (proptest)
- **Stress tests**: 24-hour soak tests with memory leak detection
- **UI tests**: Automated UI testing with Windows Application Driver

#### Graceful Degradation
- **Missing permissions**: Full functionality without admin, clear UX about limitations
- **API failures**: Fallback to alternative APIs when primary unavailable
- **Corrupted data**: Recover from malformed system data without crashes
- **Resource exhaustion**: Handle low-memory and high-load scenarios gracefully

#### Error Handling
- **No panics**: Panics forbidden in release builds (replace with Result)
- **Structured errors**: All errors use typed error enums with context
- **Recovery**: Automatic recovery from transient failures
- **User feedback**: Clear, actionable error messages for end users

#### Self-Healing
- **Configuration reset**: Auto-reset corrupted config to safe defaults
- **Cache invalidation**: Detect and recover from stale cache data
- **State recovery**: Persist and restore application state across crashes
- **Health monitoring**: Internal health checks with automatic remediation

## Performance Budgets

### Component Performance Budgets
**Hard limits enforced by CI**

| Component | Startup Time | CPU (Active) | Memory (Idle) | Memory (Active) |
|-----------|-------------|--------------|---------------|-----------------|
| Core Process Monitor | <50ms | <0.5% | <2MB | <5MB |
| Performance Graphs | <100ms | <1% | <3MB | <8MB |
| Detailed View | <80ms | <0.3% | <2MB | <4MB |
| Services Monitor | <60ms | <0.2% | <1MB | <3MB |
| Startup Manager | <120ms | <0.1% | <2MB | <4MB |
| UI Framework | <150ms | <0.2% | <5MB | <8MB |
| **Total Application** | **<500ms** | **<2%** | **<15MB** | **<25MB** |

### Allocation Budgets
- **Hot path allocation**: 0 allocations per metric collection cycle
- **UI updates**: <5 allocations per frame (amortized)
- **Process enumeration**: <1 allocation per process (via object pool)
- **Graph rendering**: <10 allocations per frame (reused buffers)

## Technology Stack

### Core Dependencies (APPROVED)
```toml
# Windows API Bindings
windows = "0.58"           # Official Microsoft windows-rs
windows-implement = "0.58" # Trait implementations
windows-core = "0.58"      # Core types

# UI Rendering
windows-ui = "0.58"        # Windows.UI.Composition
directx-rs = "*"           # DirectX 12 bindings

# Performance & Concurrency
crossbeam = "0.8"          # Lock-free data structures
parking_lot = "0.12"       # Faster mutexes than std
rayon = "1.10"             # Data parallelism (non-hot-paths only)

# Memory Management
bumpalo = "3.16"           # Bump allocator
mimalloc = "0.1"           # High-performance allocator (default)

# Error Handling & Utilities
thiserror = "1.0"          # Error derive macros
anyhow = "1.0"             # Error handling (non-library code only)

# Observability
tracing = "0.1"            # Structured logging
tracing-subscriber = "0.3" # Log collection
```

### Forbidden Dependencies
- ❌ **tokio/async-std**: No async runtime overhead (synchronous APIs only)
- ❌ **serde** (in hot paths): No serialization in performance-critical code
- ❌ **regex** (in hot paths): Pre-compile or avoid in monitoring loops
- ❌ **clap/structopt**: No CLI parsing overhead in core libraries
- ❌ **Cross-platform UI**: No gtk, qt, iced, druid, egui for main UI

### Dependency Hygiene
- **Audit quarterly**: Run `cargo audit` and address all advisories
- **Minimal versions**: Use minimum required versions for compatibility
- **Feature flags**: Disable default features, opt-in to needed features only
- **Build time**: Total clean build must complete in <2 minutes

## Architecture Principles

### Modular Design
**Clear separation of concerns with well-defined boundaries**

```
rust-task-manager/
├── core/              # Core system monitoring (no UI dependencies)
│   ├── process/       # Process enumeration and management
│   ├── performance/   # CPU, memory, disk, network metrics
│   ├── services/      # Windows services monitoring
│   └── startup/       # Startup entries management
├── windows/           # Windows API wrappers and utilities (uses windows-rs crate)
├── ui/                # UI framework and rendering
│   ├── compositor/    # Direct2D/DirectComposition integration
│   ├── controls/      # Custom UI controls
│   └── themes/        # Fluent Design implementation
├── app/               # Application entry point and orchestration
└── plugins/           # Plugin system (future extensibility)
```

**Note**: Module name is `windows/` (directory). Dependency `windows-sys` (crate) used for low-level FFI where `windows` crate lacks APIs.

#### Dependency Rules
- **Core → Windows**: Core depends only on Windows system wrappers
- **UI → Core**: UI can depend on core, never the reverse
- **App → All**: Application orchestrates all components
- **No circular deps**: Enforced by `cargo-machete` and module structure

### Plugin System Design
**FFI-compatible for future C++ plugins**

- **Stable ABI**: Use `#[repr(C)]` for all plugin interfaces
- **Version negotiation**: Plugins declare API version compatibility
- **Sandboxing**: Plugins run in separate process (future enhancement)
- **Hot reload**: Support plugin reload without app restart (dev mode only)
- **Documentation**: Full plugin API documentation with examples

### Compilation Strategy

#### Feature Flags
```toml
[features]
default = ["fluent-ui", "hardware-accel"]
fluent-ui = []           # Windows 11 Fluent Design
hardware-accel = []      # DirectX rendering (fallback: software)
developer-mode = []      # Extra diagnostics and hot reload
tracing = ["dep:tracing"] # Opt-in structured logging
nightly = []             # Nightly-only optimizations
```

#### Profile Optimization
```toml
[profile.release]
lto = "fat"              # Full link-time optimization
codegen-units = 1        # Single codegen unit for max optimization
panic = "abort"          # Smaller binary, faster panics
strip = true             # Strip symbols
opt-level = 3            # Maximum optimization

[profile.release-debug]
inherits = "release"
strip = false            # Keep symbols for profiling
debug = true
```

## Developer Experience

### Comprehensive Tracing
**Observability for debugging and optimization**

```rust
// Every hot path includes tracing spans
#[tracing::instrument(skip(self), level = "trace")]
fn collect_process_metrics(&self) -> Result<ProcessMetrics> {
    let _span = tracing::trace_span!("collect_process_metrics").entered();
    // ... implementation
}
```

#### Tracing Levels
- **ERROR**: Unrecoverable errors requiring user action
- **WARN**: Degraded functionality or potential issues
- **INFO**: Major state transitions (startup, shutdown)
- **DEBUG**: Detailed operation logging (disabled in release)
- **TRACE**: Hot path instrumentation (compile-time feature flag only)

### Performance Profiling Integration
**Built-in profiling hooks for optimization**

- **Tracy integration**: Frame markers for profiler integration
- **Flame graph export**: Generate flamegraphs from tracing data
- **Memory profiling**: Allocation tracking with `dhat-rs` integration
- **Benchmarking**: Criterion benchmarks for all hot paths

### Documentation Standards
**Every public item documented**

```rust
/// Collects CPU usage metrics for a specific process.
///
/// # Arguments
/// * `process_id` - The process ID to collect metrics for
///
/// # Returns
/// Returns `ProcessCpuMetrics` with user and kernel CPU time.
///
/// # Performance
/// This function is zero-allocation and runs in O(1) time.
/// Average execution time: ~50µs on typical hardware.
///
/// # Errors
/// Returns `Error::ProcessNotFound` if the process ID is invalid.
/// Returns `Error::AccessDenied` if insufficient privileges.
///
/// # Safety
/// This function uses `unsafe` to call Windows APIs but maintains
/// all Rust safety guarantees. See internal comments for details.
///
/// # Example
/// ```rust
/// let metrics = monitor.get_cpu_metrics(1234)?;
/// println!("CPU: {:.2}%", metrics.total_usage());
/// ```
pub fn get_cpu_metrics(&self, process_id: u32) -> Result<ProcessCpuMetrics> {
    // Implementation
}
```

### Code Review Requirements
**Quality gates before merge**

#### Required Checks
- ✅ All tests pass (unit, integration, property tests)
- ✅ Code coverage >90% for new code
- ✅ Performance benchmarks show no regressions
- ✅ `cargo clippy` with no warnings (pedantic mode)
- ✅ `cargo fmt` formatting enforced
- ✅ Security audit passes (`cargo audit`)
- ✅ Documentation complete for public APIs
- ✅ Changelog updated with user-facing changes

#### Manual Review
- Two approvals required for unsafe code
- One approval required for safe code
- Architecture changes require team discussion
- Performance-critical code requires benchmark review

## Governance

### Constitution Authority
This constitution supersedes all other development practices and guidelines. When conflicts arise, the constitution takes precedence. All pull requests must demonstrate compliance with these principles.

### Amendment Process
1. **Proposal**: Document proposed change with rationale
2. **Discussion**: Team discussion and RFC process
3. **Approval**: Unanimous team approval required
4. **Migration**: Create migration plan for existing code
5. **Ratification**: Update constitution with new version number
6. **Enforcement**: Update CI checks to enforce new requirements

### Complexity Justification
Any deviation from these principles must be justified with:
- **Performance data**: Benchmarks showing necessity
- **Alternative analysis**: Why simpler approaches won't work
- **Risk assessment**: Security and reliability implications
- **Maintenance plan**: Long-term support strategy

### Enforcement
- **CI/CD**: Automated checks enforce budgets and requirements
- **Code review**: Manual verification of principle adherence
- **Metrics dashboard**: Public dashboard tracking performance budgets
- **Quarterly audit**: Review adherence and adjust budgets as needed

---

**Version**: 1.0.0 | **Ratified**: 2025-10-21 | **Last Amended**: 2025-10-21

*This constitution defines the immutable principles of the Rust Task Manager project. All code, architecture decisions, and development processes must align with these standards to ensure we deliver the fastest, most native Windows task manager ever built in Rust.*
