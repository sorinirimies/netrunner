//! Application state and update logic for the GPUI desktop app.

use futures::StreamExt;
use gpui::{Context, SharedString};
use netrunner_core::{SpeedTestResult, TestConfig, TestEvent};

use crate::engine::spawn_speed_test;

/// Which transfer the live chart is currently tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Locating,
    Servers,
    Latency,
    Download,
    Upload,
    Done,
}

impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Phase::Idle => "Ready",
            Phase::Locating => "Detecting location…",
            Phase::Servers => "Selecting servers…",
            Phase::Latency => "Measuring latency…",
            Phase::Download => "Testing download…",
            Phase::Upload => "Testing upload…",
            Phase::Done => "Complete",
        }
    }
}

/// Maximum number of samples kept for each live chart.
pub const MAX_SAMPLES: usize = 120;

/// The root application entity.
pub struct SpeedApp {
    pub config: TestConfig,
    pub status: SharedString,
    pub phase: Phase,
    pub running: bool,

    pub download_samples: Vec<f32>,
    pub upload_samples: Vec<f32>,

    pub download_mbps: f32,
    pub upload_mbps: f32,
    pub peak_download: f32,
    pub peak_upload: f32,
    pub ping_ms: f32,

    pub location: Option<SharedString>,
    pub isp: Option<SharedString>,
    pub server: Option<SharedString>,
    pub result: Option<SpeedTestResult>,
}

impl SpeedApp {
    pub fn new() -> Self {
        Self {
            config: TestConfig::default(),
            status: "Ready to jack in.".into(),
            phase: Phase::Idle,
            running: false,
            download_samples: Vec::new(),
            upload_samples: Vec::new(),
            download_mbps: 0.0,
            upload_mbps: 0.0,
            peak_download: 0.0,
            peak_upload: 0.0,
            ping_ms: 0.0,
            location: None,
            isp: None,
            server: None,
            result: None,
        }
    }

    /// Kick off a full speed test and stream its progress into this entity.
    pub fn start(&mut self, cx: &mut Context<Self>) {
        if self.running {
            return;
        }

        // Reset state for a fresh run.
        self.running = true;
        self.phase = Phase::Locating;
        self.status = "Initialising neural interface…".into();
        self.download_samples.clear();
        self.upload_samples.clear();
        self.download_mbps = 0.0;
        self.upload_mbps = 0.0;
        self.peak_download = 0.0;
        self.peak_upload = 0.0;
        self.ping_ms = 0.0;
        self.location = None;
        self.isp = None;
        self.server = None;
        self.result = None;

        let rx = spawn_speed_test(self.config.clone());

        cx.spawn(async move |this, cx| {
            let mut rx = rx;
            while let Some(event) = rx.next().await {
                let keep_going = this
                    .update(cx, |app, cx| {
                        app.apply(event);
                        cx.notify();
                    })
                    .is_ok();
                if !keep_going {
                    break;
                }
            }
            let _ = this.update(cx, |app, cx| {
                app.running = false;
                if app.phase != Phase::Done {
                    app.phase = Phase::Done;
                }
                cx.notify();
            });
        })
        .detach();

        cx.notify();
    }

    /// Apply a single progress event to the state.
    pub fn apply(&mut self, event: TestEvent) {
        match event {
            TestEvent::Status(msg) => self.status = msg.into(),
            TestEvent::LocationDetected {
                city, country, isp, ..
            } => {
                self.location = Some(format!("{city}, {country}").into());
                self.isp = isp.map(Into::into);
                self.phase = Phase::Locating;
            }
            TestEvent::ServerPoolBuilt { .. } | TestEvent::NearbyServersFound { .. } => {
                self.phase = Phase::Servers;
            }
            TestEvent::ServersSelected { .. } => self.phase = Phase::Servers,
            TestEvent::PrimarySelected { name, location, .. } => {
                self.server = Some(format!("{name} · {location}").into());
                self.phase = Phase::Servers;
            }
            TestEvent::PhaseStarted(phase) => {
                use netrunner_core::Phase as CorePhase;
                self.phase = match phase {
                    CorePhase::Locating => Phase::Locating,
                    CorePhase::BuildingServers | CorePhase::SelectingServers => Phase::Servers,
                    CorePhase::Latency => Phase::Latency,
                    CorePhase::Download => Phase::Download,
                    CorePhase::Upload => Phase::Upload,
                    CorePhase::Jitter => self.phase,
                };
                self.status = phase.title().to_string().into();
            }
            TestEvent::DownloadSample {
                mbps, peak_mbps, ..
            } => {
                self.download_mbps = mbps as f32;
                self.peak_download = peak_mbps as f32;
                push_sample(&mut self.download_samples, mbps as f32);
            }
            TestEvent::UploadSample {
                mbps, peak_mbps, ..
            } => {
                self.upload_mbps = mbps as f32;
                self.peak_upload = peak_mbps as f32;
                push_sample(&mut self.upload_samples, mbps as f32);
            }
            TestEvent::DownloadComplete { mbps } => self.download_mbps = mbps as f32,
            TestEvent::UploadComplete { mbps } => self.upload_mbps = mbps as f32,
            TestEvent::LatencyProgress { avg_ms } | TestEvent::LatencyComplete { avg_ms } => {
                self.ping_ms = avg_ms as f32;
            }
            TestEvent::Completed(result) => {
                self.download_mbps = result.download_mbps as f32;
                self.upload_mbps = result.upload_mbps as f32;
                self.ping_ms = result.ping_ms as f32;
                self.phase = Phase::Done;
                self.status = "Test complete.".into();
                self.result = Some(*result);
            }
            TestEvent::JitterComplete { .. } | TestEvent::DiagnosticsComplete(_) => {}
        }
    }
}

impl Default for SpeedApp {
    fn default() -> Self {
        Self::new()
    }
}

fn push_sample(buf: &mut Vec<f32>, value: f32) {
    buf.push(value);
    if buf.len() > MAX_SAMPLES {
        let overflow = buf.len() - MAX_SAMPLES;
        buf.drain(0..overflow);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use netrunner_core::Phase as CorePhase;

    #[test]
    fn apply_download_sample_tracks_current_and_peak() {
        let mut app = SpeedApp::new();
        app.apply(TestEvent::PhaseStarted(CorePhase::Download));
        assert_eq!(app.phase, Phase::Download);

        app.apply(TestEvent::DownloadSample {
            mbps: 120.0,
            peak_mbps: 120.0,
            elapsed_secs: 1.0,
        });
        app.apply(TestEvent::DownloadSample {
            mbps: 80.0,
            peak_mbps: 120.0,
            elapsed_secs: 2.0,
        });

        assert_eq!(app.download_samples.len(), 2);
        assert_eq!(app.download_mbps, 80.0);
        assert_eq!(app.peak_download, 120.0);
    }

    #[test]
    fn sample_buffer_is_capped() {
        let mut app = SpeedApp::new();
        for i in 0..(MAX_SAMPLES + 50) {
            app.apply(TestEvent::UploadSample {
                mbps: i as f64,
                peak_mbps: i as f64,
                elapsed_secs: 0.0,
            });
        }
        assert_eq!(app.upload_samples.len(), MAX_SAMPLES);
        // Oldest samples were dropped, newest retained.
        assert_eq!(
            *app.upload_samples.last().unwrap(),
            (MAX_SAMPLES + 49) as f32
        );
    }

    #[test]
    fn completed_populates_result_and_metrics() {
        let mut app = SpeedApp::new();
        let result = netrunner_core::SpeedTestResult {
            download_mbps: 250.0,
            upload_mbps: 40.0,
            ping_ms: 12.0,
            ..Default::default()
        };

        app.apply(TestEvent::Completed(Box::new(result)));

        assert_eq!(app.phase, Phase::Done);
        assert!(app.result.is_some());
        assert_eq!(app.download_mbps, 250.0);
        assert_eq!(app.upload_mbps, 40.0);
        assert_eq!(app.ping_ms, 12.0);
    }

    #[test]
    fn location_event_is_formatted() {
        let mut app = SpeedApp::new();
        app.apply(TestEvent::LocationDetected {
            city: "Berlin".into(),
            country: "Germany".into(),
            isp: Some("Example ISP".into()),
            source: "ipapi.co".into(),
        });
        assert_eq!(
            app.location.as_ref().map(|s| s.to_string()),
            Some("Berlin, Germany".to_string())
        );
        assert_eq!(
            app.isp.as_ref().map(|s| s.to_string()),
            Some("Example ISP".to_string())
        );
    }
}
