//! # netrunner (GUI)
//!
//! A Zed **GPUI**-powered desktop front-end for netrunner. All network logic
//! lives in [`netrunner_core`]; this crate renders live download/upload
//! bandwidth charts while a speed test runs.
//!
//! - [`engine`] — bridges the Tokio-based core engine to GPUI's executor
//! - [`app`] — application state and progress-event handling
//! - [`view`] — the GPUI render implementation (charts, metrics)
//! - [`theme`] — the cyberpunk colour palette

// GPUI runs on macOS and Linux (see Cargo.toml).
#![cfg(any(target_os = "macos", target_os = "linux"))]

pub mod app;
pub mod engine;
pub mod theme;
pub mod view;

pub use app::SpeedApp;
