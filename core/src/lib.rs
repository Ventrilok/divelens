//! DiveLens core: the common dive format shared by every dive-log reader,
//! and the per-photo computations (depth / temperature / elapsed time).
//!
//! This crate has no UI or Tauri dependency so it can be tested on every
//! platform in CI.

pub mod profile;

/// Version of the core library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
