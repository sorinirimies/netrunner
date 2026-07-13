//! Bridges [`netrunner_core`] progress events to the cyberpunk terminal UI.
//!
//! The core engine is UI-agnostic: it streams [`TestEvent`]s over a channel.
//! This module runs a speed test and renders those events exactly the way the
//! standalone CLI always has — status lines, section headers, the live
//! download/upload bandwidth graph and the final results table.

use colored::*;
use netrunner_core::{
    ConnectionQuality, NetworkDiagnostics, NetworkDiagnosticsTool, Phase, SpeedTest,
    SpeedTestResult, TestConfig, TestEvent,
};
use tokio::sync::mpsc;

use crate::ui::{BandwidthMonitor, UI};

/// Run a full speed test and render its progress to the terminal.
///
/// Returns the final [`SpeedTestResult`]. This reproduces the classic cyberpunk
/// output, including the live bandwidth graphs, by consuming the core engine's
/// [`TestEvent`] stream.
pub async fn run_speed_test_tui(
    config: TestConfig,
) -> Result<SpeedTestResult, Box<dyn std::error::Error>> {
    let ui = UI::new(config.clone());
    let (tx, mut rx) = mpsc::unbounded_channel::<TestEvent>();

    // Live bandwidth graph state for the current transfer phase.
    let mut monitor: Option<BandwidthMonitor> = None;
    let mut first_render = true;

    // Drive the engine and render its events on the same task (the core error
    // type is not `Send`, so we can't move the future to another thread).
    let test = SpeedTest::with_events(config.clone(), Some(tx))?;
    let engine = test.run_full_test();
    tokio::pin!(engine);
    let mut result: Option<SpeedTestResult> = None;

    loop {
        tokio::select! {
            biased;
            maybe_event = rx.recv() => {
                match maybe_event {
                    Some(event) => {
                        render_speed_event(&ui, event, &mut monitor, &mut first_render).await;
                    }
                    None => break,
                }
            }
            res = &mut engine => {
                result = Some(res?);
                break;
            }
        }
    }

    // Drain any events buffered after the engine finished.
    while let Ok(event) = rx.try_recv() {
        render_speed_event(&ui, event, &mut monitor, &mut first_render).await;
    }

    result.ok_or_else(|| "speed test did not complete".into())
}

/// Render a single speed-test event to the terminal.
async fn render_speed_event(
    ui: &UI,
    event: TestEvent,
    monitor: &mut Option<BandwidthMonitor>,
    first_render: &mut bool,
) {
    match event {
        TestEvent::Status(msg) => {
            println!("{} {}", "»".bright_cyan(), msg.bright_cyan());
        }
        TestEvent::LocationDetected {
            city,
            country,
            isp,
            source,
        } => {
            println!(
                "{} {}, {} (via {})",
                "📍 Location:".bright_green(),
                city,
                country,
                source
            );
            if let Some(isp) = isp {
                println!("{} {}", "🔌 ISP:".bright_blue(), isp);
            }
        }
        TestEvent::ServerPoolBuilt { count } => {
            println!("{} {} servers in pool", "✓".bright_green(), count);
        }
        TestEvent::NearbyServersFound { count } => {
            println!("{} {} nearby servers", "✓ Found".bright_green(), count);
        }
        TestEvent::ServersSelected { servers } => {
            println!(
                "{} {} servers selected for testing",
                "✓".bright_green(),
                servers.len()
            );
            for (i, s) in servers.iter().enumerate() {
                println!(
                    "  {}. {} - {:.1} ms ({:.0} km)",
                    i + 1,
                    s.name,
                    s.latency_ms,
                    s.distance_km
                );
            }
        }
        TestEvent::PrimarySelected {
            name,
            location,
            distance_km,
        } => {
            println!(
                "{} {} ({}, {:.0} km)",
                "✓ Selected:".bright_green().bold(),
                name,
                location,
                distance_km
            );
        }
        TestEvent::PhaseStarted(phase) => {
            let _ = ui.show_section_header(phase.title());
            match phase {
                Phase::Download => {
                    *monitor = Some(
                        ui.create_bandwidth_monitor("DOWNLOAD SPEED BANDWIDTH MONITOR", "Download"),
                    );
                    *first_render = true;
                }
                Phase::Upload => {
                    *monitor = Some(
                        ui.create_bandwidth_monitor("UPLOAD SPEED BANDWIDTH MONITOR", "Upload"),
                    );
                    *first_render = true;
                }
                _ => {}
            }
        }
        TestEvent::DownloadSample { mbps, .. } | TestEvent::UploadSample { mbps, .. } => {
            if let Some(m) = monitor.as_ref() {
                m.update(mbps).await;
                if *first_render {
                    let _ = m.render_live().await;
                    *first_render = false;
                } else {
                    let _ = m.render_live_update().await;
                }
            }
        }
        TestEvent::DownloadComplete { mbps } | TestEvent::UploadComplete { mbps } => {
            if let Some(m) = monitor.take() {
                m.update(mbps).await;
                m.mark_final().await;
                let _ = m.render_live_update().await;
            }
            *first_render = true;
        }
        TestEvent::LatencyComplete { avg_ms } => {
            let explanation = if avg_ms <= 20.0 {
                "(Excellent - ideal for gaming)".bright_green().dimmed()
            } else if avg_ms <= 50.0 {
                "(Good - suitable for most activities)"
                    .bright_cyan()
                    .dimmed()
            } else if avg_ms <= 100.0 {
                "(Fair - noticeable lag)".bright_yellow().dimmed()
            } else {
                "(Poor - significant lag)".bright_red().dimmed()
            };
            println!(
                "✓ Latency: {} {}",
                format!("{:.1} ms", avg_ms).bright_cyan(),
                explanation
            );
        }
        TestEvent::Completed(result) => {
            display_results(&result);
        }
        TestEvent::LatencyProgress { .. }
        | TestEvent::JitterComplete { .. }
        | TestEvent::DiagnosticsComplete(_) => {}
    }
}

/// Run network diagnostics and render their progress to the terminal.
pub async fn run_diagnostics_tui(
    config: TestConfig,
) -> Result<NetworkDiagnostics, Box<dyn std::error::Error>> {
    let ui = UI::new(config.clone());
    let _ = ui.show_section_header("Running Network Diagnostics");

    let (tx, mut rx) = mpsc::unbounded_channel::<TestEvent>();
    let tool = NetworkDiagnosticsTool::with_events(config.clone(), Some(tx));
    let engine = tool.run_diagnostics();
    tokio::pin!(engine);
    let mut result: Option<NetworkDiagnostics> = None;

    loop {
        tokio::select! {
            biased;
            maybe_event = rx.recv() => {
                match maybe_event {
                    Some(event) => render_diag_event(event),
                    None => break,
                }
            }
            res = &mut engine => {
                result = Some(res?);
                break;
            }
        }
    }

    while let Ok(event) = rx.try_recv() {
        render_diag_event(event);
    }

    result.ok_or_else(|| "diagnostics did not complete".into())
}

fn render_diag_event(event: TestEvent) {
    match event {
        TestEvent::Status(msg) => {
            println!("{} {}", "»".bright_magenta(), msg.bright_blue());
        }
        TestEvent::DiagnosticsComplete(diag) => display_diagnostics(&diag),
        _ => {}
    }
}

/// Render the final network-diagnostics table.
fn display_diagnostics(d: &NetworkDiagnostics) {
    use prettytable::{format, Cell, Row, Table};

    println!();
    println!(
        "{}",
        ">>> CYBERNETIC NETWORK ANALYSIS <<<"
            .bright_magenta()
            .bold()
    );

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    let gateway = d
        .gateway_ip
        .map(|g| format!("{} ⚡", g))
        .unwrap_or_else(|| "❌ OFFLINE".to_string());
    table.add_row(Row::new(vec![
        Cell::new("🌐 Neural Gateway").style_spec("Fb"),
        Cell::new(&gateway),
    ]));

    let dns_servers = if d.dns_servers.is_empty() {
        "None detected".to_string()
    } else {
        d.dns_servers
            .iter()
            .map(|ip| ip.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };
    table.add_row(Row::new(vec![
        Cell::new("🧬 DNS Matrix").style_spec("Fb"),
        Cell::new(&format!("{} 🔗", dns_servers)),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("⚡ DNS Response").style_spec("Fb"),
        Cell::new(&format!("{:.2} ms", d.dns_response_time_ms)),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("🛰️ IPv6 Protocol").style_spec("Fb"),
        Cell::new(if d.is_ipv6_available {
            "✅ ACTIVE"
        } else {
            "⚠️ INACTIVE"
        }),
    ]));

    if let Some(conn) = &d.connection_type {
        table.add_row(Row::new(vec![
            Cell::new("📡 Signal Interface").style_spec("Fb"),
            Cell::new(conn),
        ]));
    }
    if let Some(iface) = &d.network_interface {
        table.add_row(Row::new(vec![
            Cell::new("🔗 Neural Port").style_spec("Fb"),
            Cell::new(&format!("⟨{}⟩", iface)),
        ]));
    }
    table.printstd();

    if !d.route_hops.is_empty() {
        println!(
            "\n{}",
            " 🌐 NEURAL PATHWAY MAPPING 🌐 ".bright_magenta().bold()
        );
        let mut trace = Table::new();
        trace.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        trace.add_row(Row::new(vec![
            Cell::new("🔗 Node").style_spec("Fb"),
            Cell::new("📍 Address").style_spec("Fb"),
            Cell::new("🏷️ Identity").style_spec("Fb"),
            Cell::new("⚡ Delay").style_spec("Fb"),
        ]));
        for hop in &d.route_hops {
            let addr = hop.address.map_or("⟨⟨⟨ ENCRYPTED ⟩⟩⟩".to_string(), |a| {
                format!("{} 🔗", a)
            });
            let hostname = hop
                .hostname
                .clone()
                .unwrap_or_else(|| "⟨ANONYMOUS⟩".to_string());
            let time = hop
                .response_time_ms
                .map_or("🔒 STEALTH".to_string(), |t| format!("{:.2} ms", t));
            trace.add_row(Row::new(vec![
                Cell::new(&format!("{:02}", hop.hop_number)),
                Cell::new(&addr),
                Cell::new(&hostname),
                Cell::new(&time),
            ]));
        }
        trace.printstd();
    }
}

/// Render the final speed-test results table.
fn display_results(result: &SpeedTestResult) {
    println!();
    println!("{}", "═".repeat(60).bright_blue());
    println!(
        "{}",
        "           SPEED TEST RESULTS           "
            .bright_yellow()
            .bold()
    );
    println!("{}", "═".repeat(60).bright_blue());
    println!();

    println!(
        "{:20} {}",
        "Download:".bright_blue().bold(),
        format!("{:.1} Mbps", result.download_mbps)
            .bright_green()
            .bold()
    );
    println!(
        "{:20} {}",
        "Upload:".bright_blue().bold(),
        format!("{:.1} Mbps", result.upload_mbps)
            .bright_green()
            .bold()
    );
    println!(
        "{:20} {}",
        "Ping:".bright_blue().bold(),
        format!("{:.1} ms", result.ping_ms).bright_cyan().bold()
    );
    println!(
        "{:20} {}",
        "Jitter:".bright_blue().bold(),
        format!("{:.1} ms", result.jitter_ms).bright_cyan()
    );

    if result.packet_loss_percent > 0.0 {
        println!(
            "{:20} {}",
            "Packet Loss:".bright_blue().bold(),
            format!("{:.1}%", result.packet_loss_percent).bright_red()
        );
    }

    println!(
        "{:20} {}",
        "Server:".bright_blue().bold(),
        result.server_location.bright_cyan()
    );

    if let Some(isp) = &result.isp {
        println!("{:20} {}", "ISP:".bright_blue().bold(), isp.bright_cyan());
    }

    let quality_colored = match result.quality {
        ConnectionQuality::Excellent | ConnectionQuality::Good => {
            format!("{}", result.quality).bright_green().bold()
        }
        ConnectionQuality::Average => format!("{}", result.quality).bright_yellow().bold(),
        _ => format!("{}", result.quality).bright_red().bold(),
    };
    println!("{:20} {}", "Quality:".bright_blue().bold(), quality_colored);

    println!();
    println!("{}", "═".repeat(60).bright_blue());
}
