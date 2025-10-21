// Global allocator: mimalloc for 2-3x allocation performance
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use task_manager::ui::window::Window;

fn main() -> windows::core::Result<()> {
    println!("Rust Task Manager v0.1.0");
    println!("Starting...");

    // Create main window
    let window = Window::new("Rust Task Manager", 1200, 800)?;
    window.show();

    println!("Window created successfully");

    // Run message loop
    window.run_message_loop()?;

    Ok(())
}
