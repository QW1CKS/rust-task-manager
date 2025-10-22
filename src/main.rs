// Global allocator: mimalloc for 2-3x allocation performance
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use task_manager::ui::window::Window;
use task_manager::util::profiling::StartupProfiler;

/// T310-T315: Startup time optimization targets
/// - Total: <500ms
/// - Window creation: <100ms
/// - D2D initialization: <150ms (T312: lazy, deferred to first paint)
/// - Process enumeration: <100ms (T313: parallel using rayon)
/// - UI layout: <50ms
/// - Monitoring setup: <100ms (T314: lazy, deferred to first update)
fn main() -> windows::core::Result<()> {
    let mut profiler = StartupProfiler::new();

    println!("Rust Task Manager v0.1.0");
    println!("Starting...");

    profiler.begin_phase("window_creation");
    // Create main window (D2D resources created lazily on first WM_PAINT)
    let mut window = Window::new("Rust Task Manager", 1200, 800)?;
    window.show();
    profiler.end_phase();

    println!("Window created successfully");

    profiler.begin_phase("d2d_initialization");
    // Initialize Direct2D renderer and process monitoring
    window.initialize_state()?;
    profiler.end_phase();

    println!("Monitoring started - showing live processes");

    profiler.begin_phase("message_loop");
    // Run message loop
    window.run_message_loop()?;
    profiler.end_phase();

    profiler.print_breakdown();

    Ok(())
}
