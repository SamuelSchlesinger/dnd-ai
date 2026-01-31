//! Shared tokio runtime for async operations.
//!
//! This module provides a lazy-initialized global tokio runtime to avoid
//! creating a new runtime for each async operation, which is wasteful and
//! can potentially fail.

use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

/// Global shared tokio runtime for all async operations.
pub static RUNTIME: Lazy<Runtime> =
    Lazy::new(|| Runtime::new().expect("Failed to create tokio runtime"));
