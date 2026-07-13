//! UI-agnostic progress events emitted by the speed-test and diagnostics engines.
//!
//! The core crate contains **no UI code**. Instead, long-running operations
//! report their progress by sending [`TestEvent`]s over a Tokio
//! [`tokio::sync::mpsc::UnboundedSender`]. Front-ends (the Ratatui CLI and the GPUI desktop
//! app) subscribe to these events and render them however they like — the CLI
//! reproduces its cyberpunk bandwidth graphs, while the GUI feeds live
//! download/upload charts.
//!
//! When no sender is supplied the engines run silently, which is exactly what
//! `--json` output and library consumers want.

use crate::types::{NetworkDiagnostics, SpeedTestResult};

/// The channel used to stream [`TestEvent`]s from the engine to a front-end.
pub type EventSender = tokio::sync::mpsc::UnboundedSender<TestEvent>;

/// A single stage of a full speed test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Detecting the caller's geographic location.
    Locating,
    /// Building the candidate server pool.
    BuildingServers,
    /// Latency-probing the pool to pick the fastest servers.
    SelectingServers,
    /// Measuring round-trip latency.
    Latency,
    /// Measuring download throughput.
    Download,
    /// Measuring upload throughput.
    Upload,
    /// Measuring jitter and packet loss.
    Jitter,
}

impl Phase {
    /// Human-readable section title for this phase.
    pub fn title(self) -> &'static str {
        match self {
            Phase::Locating => "Detecting Location",
            Phase::BuildingServers => "Building Server Pool",
            Phase::SelectingServers => "Selecting Best Servers",
            Phase::Latency => "Testing Latency",
            Phase::Download => "Testing Download Speed",
            Phase::Upload => "Testing Upload Speed",
            Phase::Jitter => "Measuring Jitter & Packet Loss",
        }
    }
}

/// A server chosen during selection, with its measured latency and distance.
#[derive(Debug, Clone)]
pub struct SelectedServer {
    pub name: String,
    pub location: String,
    pub latency_ms: f64,
    pub distance_km: f64,
}

/// A progress event emitted while running a test.
#[derive(Debug, Clone)]
pub enum TestEvent {
    /// A generic informational status line.
    Status(String),
    /// The caller's location was resolved.
    LocationDetected {
        city: String,
        country: String,
        isp: Option<String>,
        source: String,
    },
    /// The candidate server pool was built.
    ServerPoolBuilt { count: usize },
    /// Nearby servers were discovered.
    NearbyServersFound { count: usize },
    /// The best servers were selected (fastest first).
    ServersSelected { servers: Vec<SelectedServer> },
    /// The primary (fastest) server was chosen.
    PrimarySelected {
        name: String,
        location: String,
        distance_km: f64,
    },
    /// A new phase started.
    PhaseStarted(Phase),
    /// A live download-throughput sample (megabits per second).
    DownloadSample {
        mbps: f64,
        peak_mbps: f64,
        elapsed_secs: f64,
    },
    /// A live upload-throughput sample (megabits per second).
    UploadSample {
        mbps: f64,
        peak_mbps: f64,
        elapsed_secs: f64,
    },
    /// Final download throughput.
    DownloadComplete { mbps: f64 },
    /// Final upload throughput.
    UploadComplete { mbps: f64 },
    /// A running average latency update.
    LatencyProgress { avg_ms: f64 },
    /// Final measured latency.
    LatencyComplete { avg_ms: f64 },
    /// Final jitter and packet-loss.
    JitterComplete {
        jitter_ms: f64,
        packet_loss_percent: f64,
    },
    /// Diagnostics finished.
    DiagnosticsComplete(Box<NetworkDiagnostics>),
    /// The whole speed test finished.
    Completed(Box<SpeedTestResult>),
}

/// Send an event on an optional channel, ignoring send errors (a dropped
/// receiver simply means nobody is listening).
#[inline]
pub fn emit(tx: &Option<EventSender>, event: TestEvent) {
    if let Some(tx) = tx {
        let _ = tx.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_titles_are_present() {
        assert_eq!(Phase::Download.title(), "Testing Download Speed");
        assert_eq!(Phase::Upload.title(), "Testing Upload Speed");
        assert_eq!(Phase::Latency.title(), "Testing Latency");
    }

    #[test]
    fn emit_is_a_noop_without_a_sender() {
        // Should not panic when there is no receiver.
        emit(&None, TestEvent::Status("hello".into()));
    }

    #[test]
    fn emit_delivers_to_receiver() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        emit(&Some(tx), TestEvent::ServerPoolBuilt { count: 7 });
        match rx.try_recv() {
            Ok(TestEvent::ServerPoolBuilt { count }) => assert_eq!(count, 7),
            other => panic!("unexpected event: {other:?}"),
        }
    }
}
