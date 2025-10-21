//! Native High-Performance Task Manager
//!
//! Ultra-fast Windows task manager built with pure Rust.
//!
//! # Architecture
//!
//! - `core`: Platform-agnostic business logic
//! - `windows`: Windows-specific implementations
//! - `ui`: User interface layer with Direct2D
//! - `app`: Application coordination

#![warn(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod app;
pub mod core;
pub mod ui;
pub mod util;
pub mod windows;
