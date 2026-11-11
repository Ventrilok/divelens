//! DiveLens core: the common dive format shared by every dive-log reader,
//! the readers themselves, and the per-photo computations
//! (depth / temperature / elapsed time).
//!
//! This crate has no UI or Tauri dependency so it can be tested on every
//! platform in CI.

pub mod divecomputer;
pub mod error;
pub mod macdive;
pub mod model;
pub mod profile;
pub mod tags;

pub use error::{Error, Result};
pub use model::{Capabilities, Dive, DiveSource, Site};

/// Version of the core library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
