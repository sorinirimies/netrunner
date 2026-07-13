//! # netrunner-core
//!
//! Framework-free internet speed-test and network-diagnostics engine shared by
//! the `netrunner_cli` (Ratatui TUI) and `netrunner` (Zed GPUI desktop) apps,
//! and usable as a standalone library.
//!
//! | Module | What lives here |
//! |--------|-----------------|
//! | [`types`] | Domain models — results, servers, config, quality ratings, diagnostics |
//! | [`speed_test`] | Geolocation, server discovery/selection, throughput & latency measurement ([`SpeedTest`]) |
//! | [`diagnostics`] | Gateway/DNS/route/IPv6 network diagnostics ([`NetworkDiagnosticsTool`]) |
//! | [`history`] | Embedded `redb` history storage & statistics ([`HistoryStorage`]) |
//! | [`events`] | UI-agnostic progress events ([`TestEvent`]) streamed over a channel |
//!
//! This crate has **no GUI or TUI dependencies**. Progress is reported through
//! [`TestEvent`]s so any front-end can render it.
//!
//! ## Quick start
//!
//! ```no_run
//! use netrunner_core::{SpeedTest, TestConfig};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let test = SpeedTest::new(TestConfig::default())?;
//! let result = test.run_full_test().await?;
//! println!("{:.1} Mbps down / {:.1} Mbps up", result.download_mbps, result.upload_mbps);
//! # Ok(())
//! # }
//! ```

pub mod diagnostics;
pub mod events;
pub mod history;
pub mod presentation;
pub mod speed_test;
pub mod types;

// Convenience re-exports — the public surface most consumers need.
pub use diagnostics::NetworkDiagnosticsTool;
pub use events::{emit, EventSender, Phase, SelectedServer, TestEvent};
pub use history::{DbStats, HistoryStorage, SpeedTrends, TestStatistics};
pub use presentation::{palette, quality_rgb};
pub use speed_test::{GeoLocation, SpeedTest};
pub use types::{
    ConnectionQuality, DetailLevel, NetworkDiagnostics, RouteHop, ServerCapabilities,
    ServerProvider, SpeedTestResult, TestConfig, TestServer,
};
