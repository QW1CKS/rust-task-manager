//! UI Automation accessibility support (T396-T403)
//!
//! Implements Microsoft UI Automation providers for screen reader support:
//! - IRawElementProviderSimple for window
//! - IValueProvider for text inputs
//! - IInvokeProvider for buttons
//! - ISelectionProvider for process table
//! - Focus change notifications
//! - Accessible names and roles

pub mod uia;

pub use uia::*;
