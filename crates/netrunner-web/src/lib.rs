//! Pure, framework-free pieces of the netrunner web dashboard.
//!
//! The Topcoat routes live in `main.rs`; everything testable (and fuzzable) lives here:
//! HTML escaping, [`TestEvent`] → JSON mapping for the server-sent-event stream, and the
//! server-rendered dashboard page.

use netrunner_core::{quality_rgb, Phase, SpeedTestResult, TestEvent};
use serde_json::{json, Value};

/// Escape text for safe inclusion in HTML element content and quoted attribute values.
pub fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// A stable machine name for a [`Phase`], used by the browser to highlight progress.
pub fn phase_key(phase: Phase) -> &'static str {
    match phase {
        Phase::Locating => "locating",
        Phase::BuildingServers => "servers",
        Phase::SelectingServers => "selecting",
        Phase::Latency => "latency",
        Phase::Download => "download",
        Phase::Upload => "upload",
        Phase::Jitter => "jitter",
    }
}

/// CSS `#rrggbb` colour for a connection quality (same palette as the TUI and GPUI apps).
pub fn quality_hex(quality: netrunner_core::ConnectionQuality) -> String {
    let (r, g, b) = quality_rgb(quality);
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// Map an engine event to a named server-sent event: `(event name, JSON payload)`.
///
/// Returns `None` for events the dashboard does not render.
pub fn event_json(event: &TestEvent) -> Option<(&'static str, Value)> {
    Some(match event {
        TestEvent::Status(msg) => ("status", json!({ "message": msg })),
        TestEvent::LocationDetected {
            city,
            country,
            isp,
            source,
        } => (
            "location",
            json!({ "city": city, "country": country, "isp": isp, "source": source }),
        ),
        TestEvent::PrimarySelected {
            name,
            location,
            distance_km,
        } => (
            "server",
            json!({ "name": name, "location": location, "distance_km": distance_km }),
        ),
        TestEvent::PhaseStarted(phase) => (
            "phase",
            json!({ "phase": phase_key(*phase), "title": phase.title() }),
        ),
        TestEvent::DownloadSample {
            mbps,
            peak_mbps,
            elapsed_secs,
        } => (
            "download",
            json!({ "mbps": mbps, "peak_mbps": peak_mbps, "elapsed_secs": elapsed_secs }),
        ),
        TestEvent::UploadSample {
            mbps,
            peak_mbps,
            elapsed_secs,
        } => (
            "upload",
            json!({ "mbps": mbps, "peak_mbps": peak_mbps, "elapsed_secs": elapsed_secs }),
        ),
        TestEvent::DownloadComplete { mbps } => ("download_done", json!({ "mbps": mbps })),
        TestEvent::UploadComplete { mbps } => ("upload_done", json!({ "mbps": mbps })),
        TestEvent::LatencyProgress { avg_ms } | TestEvent::LatencyComplete { avg_ms } => {
            ("latency", json!({ "avg_ms": avg_ms }))
        }
        TestEvent::JitterComplete {
            jitter_ms,
            packet_loss_percent,
        } => (
            "jitter",
            json!({ "jitter_ms": jitter_ms, "packet_loss_percent": packet_loss_percent }),
        ),
        TestEvent::Completed(result) => (
            "done",
            json!({
                "result": result,
                "quality": result.quality.to_string(),
                "color": quality_hex(result.quality),
            }),
        ),
        _ => return None,
    })
}

/// Render the history table rows (server side, fully escaped).
pub fn history_rows(results: &[SpeedTestResult]) -> String {
    if results.is_empty() {
        return "<tr><td colspan=\"6\" class=\"dim\">No tests yet — run one!</td></tr>".into();
    }
    let mut out = String::new();
    for r in results {
        out.push_str(&format!(
            "<tr><td>{}</td><td>{:.1}</td><td>{:.1}</td><td>{:.0}</td>\
             <td style=\"color:{}\">{}</td><td>{}</td></tr>",
            r.timestamp.format("%Y-%m-%d %H:%M"),
            r.download_mbps,
            r.upload_mbps,
            r.ping_ms,
            quality_hex(r.quality),
            escape_html(&r.quality.to_string()),
            escape_html(&r.server_location),
        ));
    }
    out
}

const CSS: &str = include_str!("dashboard.css");
const JS: &str = include_str!("dashboard.js");

/// Render the full dashboard document.
pub fn render_index(history: &[SpeedTestResult]) -> String {
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>netrunner</title><style>{CSS}</style></head><body>\
         <main><header><h1>NET<span>RUNNER</span></h1>\
         <button id=\"run\" type=\"button\">▶ Run speed test</button></header>\
         <p id=\"status\" class=\"dim\">Idle</p>\
         <ol id=\"phases\"><li data-p=\"locating\">Locate</li><li data-p=\"selecting\">Servers</li>\
         <li data-p=\"latency\">Latency</li><li data-p=\"download\">Download</li>\
         <li data-p=\"upload\">Upload</li><li data-p=\"jitter\">Jitter</li></ol>\
         <section class=\"cards\">\
         <div class=\"card\"><h2>Download</h2><b id=\"dl\">–</b><small>Mbps</small></div>\
         <div class=\"card\"><h2>Upload</h2><b id=\"ul\">–</b><small>Mbps</small></div>\
         <div class=\"card\"><h2>Ping</h2><b id=\"ping\">–</b><small>ms</small></div>\
         <div class=\"card\"><h2>Jitter</h2><b id=\"jit\">–</b><small>ms</small></div></section>\
         <canvas id=\"chart\" width=\"900\" height=\"180\"></canvas>\
         <p id=\"verdict\"></p>\
         <h2>History</h2><table><thead><tr><th>When</th><th>↓ Mbps</th><th>↑ Mbps</th>\
         <th>Ping</th><th>Quality</th><th>Server</th></tr></thead>\
         <tbody id=\"history\">{rows}</tbody></table></main>\
         <script>{JS}</script></body></html>",
        rows = history_rows(history),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use netrunner_core::ConnectionQuality;

    #[test]
    fn escapes_html_specials() {
        assert_eq!(
            escape_html("<a href=\"x\">'&'</a>"),
            "&lt;a href=&quot;x&quot;&gt;&#39;&amp;&#39;&lt;/a&gt;"
        );
        assert_eq!(escape_html("plain"), "plain");
    }

    #[test]
    fn history_is_escaped() {
        let r = SpeedTestResult {
            loaded_latency_ms: None,
            server_location: "<script>alert(1)</script>".into(),
            ..Default::default()
        };
        let html = history_rows(&[r]);
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn empty_history_has_placeholder() {
        assert!(history_rows(&[]).contains("No tests yet"));
    }

    #[test]
    fn sample_events_map_to_json() {
        let (name, v) = event_json(&TestEvent::DownloadSample {
            mbps: 12.5,
            peak_mbps: 20.0,
            elapsed_secs: 1.0,
        })
        .unwrap();
        assert_eq!(name, "download");
        assert_eq!(v["mbps"], 12.5);
        let (name, v) = event_json(&TestEvent::PhaseStarted(Phase::Upload)).unwrap();
        assert_eq!((name, v["phase"].as_str()), ("phase", Some("upload")));
        assert!(event_json(&TestEvent::ServerPoolBuilt { count: 3 }).is_none());
    }

    #[test]
    fn completed_carries_quality_colour() {
        let result = SpeedTestResult {
            loaded_latency_ms: None,
            quality: ConnectionQuality::Excellent,
            ..Default::default()
        };
        let (name, v) = event_json(&TestEvent::Completed(Box::new(result))).unwrap();
        assert_eq!(name, "done");
        assert_eq!(v["color"], "#00ff80");
    }

    #[test]
    fn index_contains_history_and_scripts() {
        let html = render_index(&[]);
        assert!(html.starts_with("<!doctype html>"));
        assert!(html.contains("EventSource"));
        assert!(html.contains("No tests yet"));
    }
}
