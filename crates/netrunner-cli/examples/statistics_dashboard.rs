//! # Statistics Dashboard Example
//!
//! This example demonstrates the interactive statistics TUI powered by `tui-piechart`.
//! It seeds the history database with 20 realistic fake speed-test entries spanning
//! the last 30 days, so the dashboard looks great even if you have never run a real
//! test, then immediately launches the full-screen statistics viewer.
//!
//! ## Usage
//!
//! ```sh
//! cargo run --example statistics_dashboard
//! ```
//!
//! ## Features demonstrated
//!
//! - Seeding [`HistoryStorage`] with varied, realistic [`SpeedTestResult`] data
//! - Mixed [`ConnectionQuality`] ratings (Excellent → Failed) for a rich pie-chart
//! - Four European server locations: Frankfurt, Amsterdam, London, Paris
//! - Full-screen interactive TUI with pie-charts and a scrollable results table
//!
//! ## TUI controls
//!
//! | Key               | Action                                      |
//! |-------------------|---------------------------------------------|
//! | `Tab` / `→`       | Cycle to the **next** chart panel           |
//! | `←`               | Cycle to the **previous** chart panel       |
//! | `↑` / `k`         | Scroll the results table **up**             |
//! | `↓` / `j`         | Scroll the results table **down**           |
//! | `q` / `Esc`       | **Quit** the TUI and return to the terminal |
//!
//! ## Charts included
//!
//! 1. **Download speed** distribution (pie-chart, bucketed by Mbps range)
//! 2. **Upload speed** distribution (pie-chart, bucketed by Mbps range)
//! 3. **Ping / latency** distribution (pie-chart, bucketed by ms range)
//! 4. **Connection quality** breakdown (pie-chart, one slice per quality tier)
//!
//! A summary panel on the right shows aggregate statistics (avg / max / min for
//! download, upload and ping), and the bottom half of the screen lists every
//! result in a scrollable table.
//!
//! ## Notes
//!
//! - The seeded entries are written into the **same** persistent history database
//!   used by the `netrunner` binary (`~/.netrunner_cli/history.db`).  Running the
//!   example multiple times will accumulate more entries; this is harmless — the
//!   database enforces a 30-day rolling retention window automatically.
//! - No network access is required to run this example.

use chrono::{Duration, Utc};
use netrunner_cli::{show_statistics_tui, ConnectionQuality, HistoryStorage, SpeedTestResult};

// ---------------------------------------------------------------------------
// Seed data — 20 hand-crafted entries spread over the last 30 days.
// Each tuple is:
//   (days_ago, download_mbps, upload_mbps, ping_ms, jitter_ms,
//    packet_loss_pct, server_location, quality, duration_s)
// ---------------------------------------------------------------------------
type SeedEntry = (
    i64,
    f64,
    f64,
    f64,
    f64,
    f64,
    &'static str,
    ConnectionQuality,
    f64,
);

#[rustfmt::skip]
const SEED: &[SeedEntry] = &[
    // ── Excellent ────────────────────────────────────────────────────────────
    ( 1,  312.4,  48.1,   8.2,  1.1, 0.0, "Frankfurt, DE",  ConnectionQuality::Excellent, 38.2),
    ( 3,  289.7,  52.3,   9.5,  0.9, 0.0, "Amsterdam, NL",  ConnectionQuality::Excellent, 37.5),
    ( 7,  348.0,  61.8,   7.1,  0.7, 0.0, "London, UK",     ConnectionQuality::Excellent, 40.1),
    (11,  302.5,  44.9,  11.3,  1.4, 0.0, "Paris, FR",      ConnectionQuality::Excellent, 36.8),
    // ── Good ─────────────────────────────────────────────────────────────────
    ( 2,  142.6,  28.4,  22.7,  2.8, 0.0, "Frankfurt, DE",  ConnectionQuality::Good,      34.4),
    ( 5,  167.3,  31.2,  19.4,  2.3, 0.0, "Amsterdam, NL",  ConnectionQuality::Good,      35.0),
    ( 9,  131.9,  24.7,  27.1,  3.1, 0.0, "London, UK",     ConnectionQuality::Good,      33.7),
    (15,  158.0,  29.5,  24.8,  2.6, 0.0, "Paris, FR",      ConnectionQuality::Good,      34.9),
    // ── Average ──────────────────────────────────────────────────────────────
    ( 4,   62.4,  11.8,  48.3,  5.7, 0.3, "Frankfurt, DE",  ConnectionQuality::Average,   31.2),
    ( 8,   74.1,  13.2,  55.9,  6.4, 0.5, "Amsterdam, NL",  ConnectionQuality::Average,   30.5),
    (13,   58.7,  10.4,  62.0,  7.2, 0.8, "London, UK",     ConnectionQuality::Average,   29.8),
    (17,   67.2,  12.1,  51.4,  5.9, 0.4, "Paris, FR",      ConnectionQuality::Average,   31.6),
    // ── Poor ─────────────────────────────────────────────────────────────────
    ( 6,   19.3,   4.1,  98.7, 14.3, 2.1, "Frankfurt, DE",  ConnectionQuality::Poor,      28.3),
    (12,   22.8,   3.7, 118.5, 18.6, 3.4, "Amsterdam, NL",  ConnectionQuality::Poor,      27.9),
    (20,   17.6,   2.9, 131.2, 21.7, 4.7, "London, UK",     ConnectionQuality::Poor,      26.5),
    // ── Very Poor ────────────────────────────────────────────────────────────
    (10,    6.1,   1.3, 187.4, 34.2, 7.8, "Paris, FR",      ConnectionQuality::VeryPoor,  24.1),
    (22,    4.8,   0.9, 212.6, 41.5, 9.3, "Frankfurt, DE",  ConnectionQuality::VeryPoor,  22.7),
    (25,    3.2,   0.7, 248.0, 55.1,11.2, "Amsterdam, NL",  ConnectionQuality::VeryPoor,  21.3),
    // ── Failed ───────────────────────────────────────────────────────────────
    (18,    0.0,   0.0,   0.0,  0.0, 100.0, "London, UK",   ConnectionQuality::Failed,     5.0),
    (28,    0.0,   0.0,   0.0,  0.0, 100.0, "Paris, FR",    ConnectionQuality::Failed,     5.0),
];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ── Banner ────────────────────────────────────────────────────────────────
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║       NetRunner CLI — Statistics Dashboard Example           ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║                                                               ║");
    println!("║  This example seeds 20 realistic speed-test results into      ║");
    println!("║  the history database and then launches the interactive       ║");
    println!("║  statistics TUI powered by tui-piechart.                     ║");
    println!("║                                                               ║");
    println!("║  Controls once the TUI opens:                                 ║");
    println!("║    Tab / →   cycle to the next chart                         ║");
    println!("║    ←         cycle to the previous chart                     ║");
    println!("║    ↑ / k     scroll results table up                         ║");
    println!("║    ↓ / j     scroll results table down                       ║");
    println!("║    q / Esc   quit and return to the terminal                 ║");
    println!("║                                                               ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    // ── Seed history ─────────────────────────────────────────────────────────
    println!(
        "📦 Seeding history database with {} demo entries...",
        SEED.len()
    );

    {
        let storage = HistoryStorage::new()?;

        // Wipe any stale records (e.g. old bincode bytes from a previous version)
        // before inserting fresh postcard-encoded demo data.
        storage.clear_history()?;

        let now = Utc::now();

        for (
            days_ago,
            download_mbps,
            upload_mbps,
            ping_ms,
            jitter_ms,
            packet_loss_percent,
            server_location,
            quality,
            test_duration_seconds,
        ) in SEED
        {
            let result = SpeedTestResult {
                timestamp: now - Duration::days(*days_ago),
                download_mbps: *download_mbps,
                upload_mbps: *upload_mbps,
                ping_ms: *ping_ms,
                jitter_ms: *jitter_ms,
                packet_loss_percent: *packet_loss_percent,
                server_location: server_location.to_string(),
                server_ip: None,
                client_ip: None,
                quality: *quality,
                test_duration_seconds: *test_duration_seconds,
                isp: Some("Demo ISP".to_string()),
            };

            storage.save_result(&result)?;
        }

        println!("✓ {} entries written to history database", SEED.len());
    }
    // `storage` is dropped here, releasing the redb write lock before the TUI
    // opens its own independent handle to the same database.
    println!();
    println!("🚀 Launching statistics TUI — press q or Esc to exit...");
    println!();

    // ── Launch TUI ────────────────────────────────────────────────────────────
    show_statistics_tui()?;

    // ── Post-exit message ─────────────────────────────────────────────────────
    println!();
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║                      Demo complete!                          ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║                                                               ║");
    println!("║  The 20 seeded entries remain in your history database.       ║");
    println!("║  Run  netrunner history  to browse them in the terminal, or   ║");
    println!("║  run  netrunner stats    to reopen this TUI any time.         ║");
    println!("║                                                               ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();

    Ok(())
}
