//! # Continuous Monitoring Example
//!
//! This example demonstrates how to use NetRunner for continuous network monitoring.
//! It runs periodic speed tests and tracks network performance over time.
//!
//! Usage:
//!   cargo run --example continuous_monitoring
//!
//! Features demonstrated:
//! - Periodic speed testing
//! - Real-time monitoring
//! - Performance alerts
//! - Data logging
//! - Trend detection
//! - Uptime tracking

use chrono::{DateTime, Utc};
use netrunner_cli::{ConnectionQuality, DetailLevel, HistoryStorage, SpeedTest, TestConfig};
use std::fmt;
use std::time::Duration;
use tokio::time;

/// Monitoring configuration
struct MonitorConfig {
    /// Interval between tests (in seconds)
    interval_seconds: u64,
    /// Minimum acceptable download speed (Mbps)
    min_download_mbps: f64,
    /// Minimum acceptable upload speed (Mbps)
    min_upload_mbps: f64,
    /// Maximum acceptable ping (ms)
    max_ping_ms: f64,
    /// Enable alerts
    alerts_enabled: bool,
    /// Log file path
    log_file: Option<String>,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 300, // 5 minutes
            min_download_mbps: 50.0,
            min_upload_mbps: 10.0,
            max_ping_ms: 50.0,
            alerts_enabled: true,
            log_file: Some("network_monitor.log".to_string()),
        }
    }
}

/// Performance alert types
#[derive(Debug)]
enum Alert {
    SlowDownload(f64),
    SlowUpload(f64),
    HighLatency(f64),
    QualityDegraded(ConnectionQuality),
    TestFailed(String),
}

impl fmt::Display for Alert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Alert::SlowDownload(speed) => {
                write!(f, "⚠️  Download speed below threshold: {:.2} Mbps", speed)
            }
            Alert::SlowUpload(speed) => {
                write!(f, "⚠️  Upload speed below threshold: {:.2} Mbps", speed)
            }
            Alert::HighLatency(ping) => {
                write!(f, "⚠️  Latency above threshold: {:.2} ms", ping)
            }
            Alert::QualityDegraded(quality) => {
                write!(f, "⚠️  Connection quality degraded: {:?}", quality)
            }
            Alert::TestFailed(reason) => {
                write!(f, "❌ Speed test failed: {}", reason)
            }
        }
    }
}

/// Monitoring statistics
#[derive(Debug, Default)]
struct MonitoringStats {
    total_tests: u64,
    successful_tests: u64,
    failed_tests: u64,
    alerts_triggered: u64,
    total_downtime_seconds: u64,
    start_time: Option<DateTime<Utc>>,
}

impl MonitoringStats {
    fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            return 0.0;
        }
        (self.successful_tests as f64 / self.total_tests as f64) * 100.0
    }

    fn uptime_percentage(&self, elapsed_seconds: u64) -> f64 {
        if elapsed_seconds == 0 {
            return 100.0;
        }
        let uptime = elapsed_seconds - self.total_downtime_seconds;
        (uptime as f64 / elapsed_seconds as f64) * 100.0
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install crypto provider");

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║      NetRunner CLI - Continuous Monitoring Example       ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    // Initialize monitoring configuration
    let monitor_config = MonitorConfig {
        interval_seconds: 60, // 1 minute for demo (use 300+ for production)
        min_download_mbps: 50.0,
        min_upload_mbps: 10.0,
        max_ping_ms: 50.0,
        alerts_enabled: true,
        log_file: Some("network_monitor.log".to_string()),
    };

    println!("⚙️  Monitoring Configuration:");
    println!(
        "   • Test Interval: {} seconds",
        monitor_config.interval_seconds
    );
    println!(
        "   • Min Download:  {:.1} Mbps",
        monitor_config.min_download_mbps
    );
    println!(
        "   • Min Upload:    {:.1} Mbps",
        monitor_config.min_upload_mbps
    );
    println!("   • Max Latency:   {:.1} ms", monitor_config.max_ping_ms);
    println!(
        "   • Alerts:        {}",
        if monitor_config.alerts_enabled {
            "Enabled"
        } else {
            "Disabled"
        }
    );
    if let Some(ref log) = monitor_config.log_file {
        println!("   • Log File:      {}", log);
    }
    println!();

    // Initialize history storage
    let history = HistoryStorage::new()?;
    let mut stats = MonitoringStats {
        start_time: Some(Utc::now()),
        ..Default::default()
    };

    println!("🚀 Starting continuous monitoring...");
    println!("   Press Ctrl+C to stop");
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!();

    // Setup speed test configuration
    let test_config = TestConfig {
        server_url: "https://speed.cloudflare.com".to_string(),
        test_size_mb: 50,
        timeout_seconds: 60,
        json_output: true, // Suppress UI output for cleaner monitoring
        animation_enabled: false,
        detail_level: DetailLevel::Standard,
        max_servers: 2,
    };

    // Main monitoring loop
    let mut test_number = 1;
    let mut interval = time::interval(Duration::from_secs(monitor_config.interval_seconds));

    loop {
        interval.tick().await;

        let test_time = Utc::now();
        println!(
            "📊 Test #{} - {}",
            test_number,
            test_time.format("%Y-%m-%d %H:%M:%S")
        );
        println!("───────────────────────────────────────────────────────────");

        stats.total_tests += 1;

        // Run speed test
        let speed_test = SpeedTest::new(test_config.clone())?;

        match speed_test.run_full_test().await {
            Ok(result) => {
                stats.successful_tests += 1;

                // Display results
                println!("   ↓ Download: {:.2} Mbps", result.download_mbps);
                println!("   ↑ Upload:   {:.2} Mbps", result.upload_mbps);
                println!("   📡 Ping:    {:.2} ms", result.ping_ms);
                println!("   ⚡ Quality:  {:?}", result.quality);

                // Save to history
                if let Err(e) = history.save_result(&result) {
                    eprintln!("   ⚠️  Failed to save to history: {}", e);
                }

                // Check for alerts
                let mut alerts = Vec::new();

                if result.download_mbps < monitor_config.min_download_mbps {
                    alerts.push(Alert::SlowDownload(result.download_mbps));
                }

                if result.upload_mbps < monitor_config.min_upload_mbps {
                    alerts.push(Alert::SlowUpload(result.upload_mbps));
                }

                if result.ping_ms > monitor_config.max_ping_ms {
                    alerts.push(Alert::HighLatency(result.ping_ms));
                }

                if matches!(
                    result.quality,
                    ConnectionQuality::Poor
                        | ConnectionQuality::VeryPoor
                        | ConnectionQuality::Failed
                ) {
                    alerts.push(Alert::QualityDegraded(result.quality));
                }

                // Display alerts
                if !alerts.is_empty() && monitor_config.alerts_enabled {
                    println!();
                    println!("   🚨 ALERTS:");
                    for alert in &alerts {
                        println!("      {}", alert);
                        stats.alerts_triggered += 1;
                    }
                }

                // Log to file if configured
                if let Some(ref log_file) = monitor_config.log_file {
                    let log_entry = format!(
                        "{},{:.2},{:.2},{:.2},{:?},{}\n",
                        test_time.to_rfc3339(),
                        result.download_mbps,
                        result.upload_mbps,
                        result.ping_ms,
                        result.quality,
                        if alerts.is_empty() { "OK" } else { "ALERT" }
                    );

                    if let Err(e) = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(log_file)
                        .and_then(|mut file| {
                            std::io::Write::write_all(&mut file, log_entry.as_bytes())
                        })
                    {
                        eprintln!("   ⚠️  Failed to write to log: {}", e);
                    }
                }

                println!("   ✓ Test completed successfully");
            }
            Err(e) => {
                stats.failed_tests += 1;
                stats.total_downtime_seconds += monitor_config.interval_seconds;

                println!("   ❌ Test failed: {}", e);

                if monitor_config.alerts_enabled {
                    let alert = Alert::TestFailed(e.to_string());
                    println!("   🚨 {}", alert);
                    stats.alerts_triggered += 1;
                }

                // Log failure
                if let Some(ref log_file) = monitor_config.log_file {
                    let log_entry = format!(
                        "{},FAILED,FAILED,FAILED,Failed,\"{}\"\n",
                        test_time.to_rfc3339(),
                        e.to_string().replace("\"", "")
                    );

                    let _ = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(log_file)
                        .and_then(|mut file| {
                            std::io::Write::write_all(&mut file, log_entry.as_bytes())
                        });
                }
            }
        }

        println!();

        // Display monitoring statistics every 5 tests
        if test_number % 5 == 0 {
            display_statistics(&stats, &monitor_config);
        }

        test_number += 1;
    }
}

fn display_statistics(stats: &MonitoringStats, _config: &MonitorConfig) {
    println!("═══════════════════════════════════════════════════════════");
    println!("📈 Monitoring Statistics");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    if let Some(start_time) = stats.start_time {
        let elapsed = Utc::now().signed_duration_since(start_time).num_seconds() as u64;
        let hours = elapsed / 3600;
        let minutes = (elapsed % 3600) / 60;

        println!("   Runtime: {}h {}m", hours, minutes);
        println!("   Uptime:  {:.2}%", stats.uptime_percentage(elapsed));
        println!();
    }

    println!("   Total Tests:      {}", stats.total_tests);
    println!("   Successful:       {}", stats.successful_tests);
    println!("   Failed:           {}", stats.failed_tests);
    println!("   Success Rate:     {:.2}%", stats.success_rate());
    println!("   Alerts Triggered: {}", stats.alerts_triggered);
    println!();
    println!("═══════════════════════════════════════════════════════════");
    println!();
}

// Additional utility functions for production use:
//
// 1. Email/Slack Alerts:
// ```rust
// async fn send_alert(alert: &Alert) {
//     // Implement email/Slack notification
//     // Use reqwest to send webhooks
// }
// ```
//
// 2. Prometheus Metrics Export:
// ```rust
// fn export_prometheus_metrics(result: &SpeedTestResult) -> String {
//     format!(
//         "network_download_mbps {}\nnetwork_upload_mbps {}\nnetwork_ping_ms {}",
//         result.download_mbps, result.upload_mbps, result.ping_ms
//     )
// }
// ```
//
// 3. InfluxDB Integration:
// ```rust
// async fn write_to_influxdb(result: &SpeedTestResult) {
//     // Use influxdb crate to write time-series data
// }
// ```
//
// 4. Grafana Dashboard:
// - Query the history database or log files
// - Create visualizations for download/upload/ping over time
// - Set up alerts based on thresholds
//
// 5. Health Check Endpoint:
// ```rust
// #[tokio::main]
// async fn health_server() {
//     // Expose HTTP endpoint for health checks
//     // Return last test results and uptime
// }
// ```
