// Global allocator: mimalloc for 2-3x allocation performance
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    // TODO: Initialize application
    println!("Rust Task Manager v0.1.0");
    println!("Starting...");

    // Placeholder for Phase 2 window creation
    std::process::exit(0);
}
