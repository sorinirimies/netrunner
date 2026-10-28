//! Full-screen live speed-test dashboard (ratatui).
//!
//! Consumes the core engine's [`TestEvent`] stream and draws, every frame:
//!
//! - a `tui-spinner` activity header (phase tracker + animated bar),
//! - a `tui-chart` [`LineChart`] of download/upload throughput over time,
//! - `tui-chart` [`Gauge`]s for the current speeds and a latency [`Sparkline`],
//!
//! all themed through `tui-nuances` (`NETRUNNER_THEME`, `t`/`T` to cycle live).
//! Falls back to the classic line output when stdout is not a terminal or
//! animations are disabled (see [`crate::render::run_speed_test_tui`]).

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use netrunner_core::{Phase, SpeedTest, SpeedTestResult, TestConfig, TestEvent};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tokio::sync::mpsc;
use tui_chart::{Axis, Gauge, LegendPosition, LineChart, LineMode, Series, Sparkline};
use tui_nuances::Theme;
use tui_spinner::{BarSpinner, FluxSpinner};

/// Phases shown in the tracker, in test order.
const TRACKED: [(Phase, &str); 6] = [
    (Phase::Locating, "Locate"),
    (Phase::SelectingServers, "Servers"),
    (Phase::Latency, "Latency"),
    (Phase::Jitter, "Jitter"),
    (Phase::Download, "Download"),
    (Phase::Upload, "Upload"),
];

/// Everything the dashboard shows; updated purely from [`TestEvent`]s.
#[derive(Debug)]
pub struct LiveState {
    pub(crate) tick: u64,
    pub(crate) theme: Theme,
    phase: Option<Phase>,
    status: String,
    location: Option<String>,
    server: Option<String>,
    download: Vec<(f64, f64)>,
    upload: Vec<(f64, f64)>,
    download_peak: f64,
    upload_peak: f64,
    ping: Vec<f64>,
    jitter_ms: Option<f64>,
    loaded: Vec<f64>,
    pub(crate) warning: Option<String>,
    pub(crate) done: bool,
    pub(crate) finished: bool,
    summary: Option<String>,
}

impl LiveState {
    pub fn new(theme: Theme) -> Self {
        Self {
            tick: 0,
            theme,
            phase: None,
            status: "Initialising…".into(),
            location: None,
            server: None,
            download: Vec::new(),
            upload: Vec::new(),
            download_peak: 0.0,
            upload_peak: 0.0,
            ping: Vec::new(),
            jitter_ms: None,
            loaded: Vec::new(),
            warning: None,
            done: false,
            finished: false,
            summary: None,
        }
    }

    /// Fold one engine event into the state.
    pub fn apply(&mut self, event: TestEvent) {
        match event {
            TestEvent::Status(msg) => self.status = msg,
            TestEvent::LocationDetected { city, country, .. } => {
                self.location = Some(format!("{city}, {country}"));
            }
            TestEvent::PrimarySelected { name, location, .. } => {
                self.server = Some(format!("{name} · {location}"));
            }
            TestEvent::PhaseStarted(phase) => {
                self.status = phase.title().to_string();
                self.phase = Some(phase);
            }
            TestEvent::DownloadSample {
                mbps,
                peak_mbps,
                elapsed_secs,
            } => {
                self.download.push((elapsed_secs, mbps));
                self.download_peak = self.download_peak.max(peak_mbps).max(mbps);
            }
            TestEvent::UploadSample {
                mbps,
                peak_mbps,
                elapsed_secs,
            } => {
                self.upload.push((elapsed_secs, mbps));
                self.upload_peak = self.upload_peak.max(peak_mbps).max(mbps);
            }
            TestEvent::LatencyProgress { avg_ms } | TestEvent::LatencyComplete { avg_ms } => {
                self.ping.push(avg_ms);
            }
            TestEvent::LoadedLatency { ms } => self.loaded.push(ms),
            TestEvent::Warning(msg) => self.warning = Some(msg),
            TestEvent::JitterComplete { jitter_ms, .. } => self.jitter_ms = Some(jitter_ms),
            TestEvent::Completed(r) => {
                self.done = true;
                self.summary = Some(format!(
                    "{:?} connection · loss {:.1}% · bufferbloat {} · {:.0}s{}",
                    r.quality,
                    r.packet_loss_percent,
                    r.bufferbloat_grade().unwrap_or("n/a"),
                    r.test_duration_seconds,
                    r.isp
                        .as_deref()
                        .map_or(String::new(), |i| format!(" · {i}")),
                ));
            }
            _ => {}
        }
    }

    fn current_download(&self) -> f64 {
        self.download.last().map_or(0.0, |p| p.1)
    }

    fn current_upload(&self) -> f64 {
        self.upload.last().map_or(0.0, |p| p.1)
    }
}

fn panel<'a>(title: &'a str, theme: &Theme) -> Block<'a> {
    Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
}

/// Draw one frame.
pub fn draw(frame: &mut Frame, s: &LiveState) {
    let t = &s.theme;
    let area = frame.area();
    frame.render_widget(
        Block::default().style(Style::default().bg(t.background.unwrap_or(Color::Reset))),
        area,
    );
    let [body, footer] = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(area);
    draw_dashboard(frame, body, s);

    let hint = hint_line(s);
    frame.render_widget(Paragraph::new(hint).alignment(Alignment::Center), footer);
}

fn hint_line(s: &LiveState) -> Line<'static> {
    let t = &s.theme;
    Line::from(vec![
        Span::styled(
            if s.finished { "any key" } else { "q/Esc" },
            Style::default().fg(t.warning).bold(),
        ),
        Span::raw(if s.finished {
            " continue   "
        } else {
            " cancel   "
        }),
        Span::styled("t/T", Style::default().fg(t.warning).bold()),
        Span::raw(format!(" theme ({})", t.name)),
    ])
}

/// The dashboard proper (header, activity bar, chart, meters) in `area`.
/// Shared by the one-shot live screen and the interactive workbench.
pub fn draw_dashboard(frame: &mut Frame, area: Rect, s: &LiveState) {
    let t = &s.theme;
    let [header, bar, chart, meters] = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Min(6),
        Constraint::Length(5),
    ])
    .areas(area);

    draw_header(frame, header, s);
    frame.render_widget(
        BarSpinner::new(s.tick / 2)
            .arc_color(t.accent)
            .dim_color(t.muted),
        bar,
    );
    draw_chart(frame, chart, s);
    draw_meters(frame, meters, s);
}

fn draw_header(frame: &mut Frame, area: Rect, s: &LiveState) {
    let t = &s.theme;
    let block = panel("NETRUNNER // LIVE", t);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let [spin, text] = Layout::horizontal([Constraint::Length(2), Constraint::Min(0)]).areas(inner);
    if !s.done {
        frame.render_widget(FluxSpinner::new(s.tick / 2).color(t.accent), spin);
    } else {
        frame.render_widget(
            Paragraph::new("✔").style(Style::default().fg(t.success)),
            spin,
        );
    }

    let current = s
        .phase
        .map(|p| match p {
            Phase::BuildingServers => Phase::SelectingServers,
            p => p,
        })
        .and_then(|p| TRACKED.iter().position(|(tp, _)| *tp == p));
    let mut spans = Vec::new();
    for (i, (_, label)) in TRACKED.iter().enumerate() {
        let style = match current {
            _ if s.done => Style::default().fg(t.success),
            Some(c) if i == c => Style::default()
                .fg(t.accent)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
            Some(c) if i < c => Style::default().fg(t.success),
            _ => Style::default().fg(t.muted),
        };
        spans.push(Span::styled(format!(" {label} "), style));
        spans.push(Span::styled("›", Style::default().fg(t.muted)));
    }
    spans.pop();
    let mut lines = vec![Line::from(spans)];
    let mut info = s.summary.clone().unwrap_or_else(|| s.status.clone());
    if let Some(w) = &s.warning {
        info = format!("⚠ {w}  ·  {info}");
    }
    if let Some(loc) = &s.location {
        info = format!("{info}  ·  📍 {loc}");
    }
    if let Some(srv) = &s.server {
        info = format!("{info}  ·  🖧 {srv}");
    }
    lines.push(Line::styled(info, Style::default().fg(t.muted)));
    frame.render_widget(Paragraph::new(lines), text);
}

fn draw_chart(frame: &mut Frame, area: Rect, s: &LiveState) {
    let t = &s.theme;
    let chart = LineChart::new(vec![
        Series::new("Download")
            .color(t.success)
            .data(s.download.iter().copied()),
        Series::new("Upload")
            .color(t.secondary)
            .data(s.upload.iter().copied()),
    ])
    .mode(LineMode::Area)
    .legend(LegendPosition::TopLeft)
    .x_axis(Axis::new().title("seconds"))
    .y_axis(Axis::new().title("Mbps"))
    .theme(t.clone())
    .block(panel("THROUGHPUT", t));
    frame.render_widget(&chart, area);
}

fn draw_meters(frame: &mut Frame, area: Rect, s: &LiveState) {
    let t = &s.theme;
    let [dl, ul, ping] = Layout::horizontal([Constraint::Ratio(1, 3); 3]).areas(area);

    let speed = |frame: &mut Frame, area: Rect, title: &str, now: f64, peak: f64| {
        let block = panel(title, t);
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let rows = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(inner);
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("{now:.1} Mbps"),
                    Style::default().fg(t.foreground).bold(),
                ),
                Span::styled(format!("   peak {peak:.1}"), Style::default().fg(t.muted)),
            ])),
            rows[0],
        );
        frame.render_widget(
            Gauge::new(now)
                .range(0.0, nice_ceiling(peak))
                .theme(t.clone()),
            rows[1],
        );
    };
    speed(
        frame,
        dl,
        "⬇ DOWNLOAD",
        s.current_download(),
        s.download_peak,
    );
    speed(frame, ul, "⬆ UPLOAD", s.current_upload(), s.upload_peak);

    let block = panel("⚡ LATENCY", t);
    let inner = block.inner(ping);
    frame.render_widget(block, ping);
    let rows = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(inner);
    let last = s.ping.last().copied().unwrap_or(0.0);
    let mut jitter = s
        .jitter_ms
        .map_or(String::new(), |j| format!("   jitter {j:.1} ms"));
    if !s.loaded.is_empty() {
        let mut l = s.loaded.clone();
        l.sort_by(|a, b| a.total_cmp(b));
        let med = l[l.len() / 2];
        let idle = s.ping.first().copied().unwrap_or(0.0);
        jitter.push_str(&format!(
            "   loaded {med:.0} ms ({})",
            netrunner_core::bufferbloat_grade((med - idle).max(0.0))
        ));
    }
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{last:.0} ms"),
                Style::default().fg(t.foreground).bold(),
            ),
            Span::styled(jitter, Style::default().fg(t.muted)),
        ])),
        rows[0],
    );
    frame.render_widget(
        Sparkline::new(s.ping.iter().chain(s.loaded.iter()).copied())
            .color(t.warning)
            .theme(t.clone()),
        rows[1],
    );
}

/// Round a peak up to a "nice" gauge ceiling (10, 25, 50, 100, 250, 500, 1000…).
pub fn nice_ceiling(peak: f64) -> f64 {
    const STEPS: [f64; 4] = [1.0, 2.5, 5.0, 10.0];
    if !peak.is_finite() || peak <= 10.0 {
        return 10.0;
    }
    let mag = 10f64.powf(peak.log10().floor());
    STEPS
        .iter()
        .map(|m| m * mag)
        .find(|c| *c >= peak)
        .unwrap_or(mag * 10.0)
}

/// Restores the terminal on drop, even on error/panic paths.
struct TerminalGuard;
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

fn initial_theme() -> Theme {
    std::env::var("NETRUNNER_THEME")
        .ok()
        .and_then(|n| Theme::by_name(&n))
        .or_else(|| Theme::by_name("cyberpunk"))
        .unwrap_or_default()
}

/// Run a speed test with the live full-screen dashboard.
pub async fn run_speed_test_live(
    config: TestConfig,
) -> Result<SpeedTestResult, Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::unbounded_channel::<TestEvent>();
    let test = SpeedTest::with_events(config, Some(tx))?;
    let engine = test.run_full_test();
    tokio::pin!(engine);

    let mut terminal = ratatui::init();
    let _guard = TerminalGuard;
    let mut state = LiveState::new(initial_theme());
    let mut frames = tokio::time::interval(Duration::from_millis(50));
    let mut result: Option<SpeedTestResult> = None;
    let mut engine_done = false;

    loop {
        tokio::select! {
            biased;
            ev = rx.recv() => {
                match ev {
                    Some(ev) => state.apply(ev),
                    None => engine_done = true,
                }
            }
            res = &mut engine, if result.is_none() && !engine_done => {
                result = Some(res?);
            }
            _ = frames.tick() => {
                state.tick += 1;
                terminal.draw(|f| draw(f, &state))?;
                while event::poll(Duration::ZERO)? {
                    if let Event::Key(k) = event::read()? {
                        if k.kind != KeyEventKind::Press { continue; }
                        match k.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Err("cancelled".into()),
                            KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                                return Err("cancelled".into())
                            }
                            KeyCode::Char('t') => state.theme = state.theme.next(),
                            KeyCode::Char('T') => state.theme = state.theme.prev(),
                            _ => {}
                        }
                    }
                }
            }
        }
        if result.is_some() {
            // Drain buffered events, show the finished frame briefly, then leave.
            while let Ok(ev) = rx.try_recv() {
                state.apply(ev);
            }
            state.done = true;
            terminal.draw(|f| draw(f, &state))?;
            state.finished = true;
            terminal.draw(|f| draw(f, &state))?;
            // Keep the finished chart on screen until a key is pressed.
            loop {
                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(k) = event::read()? {
                        if k.kind == KeyEventKind::Press {
                            break;
                        }
                    }
                }
                state.tick += 1;
                terminal.draw(|f| draw(f, &state))?;
            }
            break;
        }
    }
    drop(terminal);
    io::Write::flush(&mut io::stdout())?;
    let result: SpeedTestResult = result.ok_or("speed test did not complete")?;
    drop(_guard);
    crate::render::display_results(&result);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn render(state: &LiveState, w: u16, h: u16) -> String {
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        term.draw(|f| draw(f, state)).unwrap();
        term.backend().to_string()
    }

    #[test]
    fn nice_ceiling_rounds_up() {
        assert_eq!(nice_ceiling(0.0), 10.0);
        assert_eq!(nice_ceiling(f64::NAN), 10.0);
        assert_eq!(nice_ceiling(42.0), 50.0);
        assert_eq!(nice_ceiling(51.0), 100.0);
        assert_eq!(nice_ceiling(260.0), 500.0);
        assert_eq!(nice_ceiling(940.0), 1000.0);
    }

    #[test]
    fn events_update_state() {
        let mut s = LiveState::new(Theme::default());
        s.apply(TestEvent::PhaseStarted(Phase::Download));
        s.apply(TestEvent::DownloadSample {
            mbps: 80.0,
            peak_mbps: 95.0,
            elapsed_secs: 1.0,
        });
        s.apply(TestEvent::LatencyComplete { avg_ms: 12.0 });
        assert_eq!(s.phase, Some(Phase::Download));
        assert_eq!(s.current_download(), 80.0);
        assert_eq!(s.download_peak, 95.0);
        assert_eq!(s.ping, vec![12.0]);
    }

    #[test]
    fn draws_empty_and_populated_without_panic() {
        let mut s = LiveState::new(Theme::default());
        for (w, h) in [(120, 40), (80, 24), (40, 12), (10, 4)] {
            render(&s, w, h);
        }
        for i in 0..30 {
            s.apply(TestEvent::DownloadSample {
                mbps: 50.0 + i as f64,
                peak_mbps: 80.0,
                elapsed_secs: i as f64 * 0.2,
            });
            s.apply(TestEvent::UploadSample {
                mbps: 20.0 + i as f64,
                peak_mbps: 40.0,
                elapsed_secs: i as f64 * 0.2,
            });
            s.apply(TestEvent::LatencyProgress {
                avg_ms: 10.0 + i as f64,
            });
        }
        let out = render(&s, 120, 40);
        assert!(out.contains("THROUGHPUT") && out.contains("DOWNLOAD"));
    }
}
