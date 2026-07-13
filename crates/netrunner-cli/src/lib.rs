//! # netrunner_cli
//!
//! The terminal front-end for **netrunner** — a cyberpunk internet speed-test
//! and network-diagnostics tool. All network logic lives in
//! [`netrunner_core`]; this crate owns the Ratatui/`indicatif`/`colored`
//! presentation layer:
//!
//! - [`ui`] — spinners, banners and the live bandwidth graph
//! - [`intro`] — the animated glowing-logo splash screen
//! - [`logo`] — the `NETRUNNER` logo widget
//! - [`stats_ui`] — the full-screen statistics dashboard with pie charts
//! - [`render`] — bridges [`netrunner_core`] progress events to the terminal UI
//!
//! The core engine reports progress through
//! [`netrunner_core::TestEvent`]s; [`render`] subscribes to that stream and
//! reproduces the cyberpunk output (including the download/upload bandwidth
//! graphs).

pub mod intro;
pub mod logo;
pub mod render;
pub mod stats_ui;
pub mod ui;

// Re-export the logo (part of this crate's public API) and the core surface
// so existing consumers keep working after the workspace split.
pub use intro::{show_intro, show_simple_intro};
pub use logo::{NetrunnerLogo, NetrunnerLogoSize};
pub use render::{run_diagnostics_tui, run_speed_test_tui};
pub use stats_ui::show_statistics_tui;
pub use ui::{BandwidthMonitor, UI};

// Convenience re-exports from the core crate.
pub use netrunner_core::{
    ConnectionQuality, DetailLevel, GeoLocation, HistoryStorage, NetworkDiagnostics,
    NetworkDiagnosticsTool, Phase, SpeedTest, SpeedTestResult, TestConfig, TestEvent,
    TestStatistics,
};
