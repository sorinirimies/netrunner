//! Statistics TUI - Visualizes historical speed test statistics using tui-piechart
//!
//! Renders an interactive full-screen TUI with:
//! - Pie charts for download/upload/ping distribution breakdowns
//! - Summary statistics panel
//! - Recent results table
//! - Keyboard navigation

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Padding, Paragraph, Row, Table},
    Terminal,
};
use std::io;
use std::time::Duration;
use tui_chart::{
    Axis, Gauge, LegendPosition as ChartLegend, LineChart, LineMode, Series, Sparkline,
};
use tui_piechart::{symbols, LegendAlignment, LegendLayout, LegendPosition, PieChart, PieSlice};

use netrunner_core::{HistoryStorage, SpeedTestResult, TestStatistics};

// ── Cyberpunk colour palette ─────────────────────────────────────────────────
/// Colours of the active `tui-nuances` theme (set before every draw).
#[derive(Clone, Copy)]
struct Pal {
    cyan: Color,
    magenta: Color,
    green: Color,
    yellow: Color,
    orange: Color,
    red: Color,
    blue: Color,
    dim: Color,
    panel_bg: Color,
}

impl Pal {
    fn of(t: &tui_nuances::Theme) -> Self {
        Self {
            cyan: t.accent,
            magenta: t.secondary,
            green: t.success,
            yellow: t.warning,
            orange: tui_nuances::color::mix(t.warning, t.error, 0.5),
            red: t.error,
            blue: t.info,
            dim: t.muted,
            panel_bg: t.background.unwrap_or(Color::Reset),
        }
    }
}

thread_local! {
    static PAL: std::cell::Cell<Pal> = std::cell::Cell::new(Pal::of(&initial_theme()));
}

fn pal() -> Pal {
    PAL.with(std::cell::Cell::get)
}

/// `NETRUNNER_THEME` if set and valid, otherwise the neon `cyberpunk` theme.
fn initial_theme() -> tui_nuances::Theme {
    std::env::var("NETRUNNER_THEME")
        .ok()
        .and_then(|n| tui_nuances::Theme::by_name(&n))
        .or_else(|| tui_nuances::Theme::by_name("cyberpunk"))
        .unwrap_or_default()
}

/// Which chart panel is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Download,
    Upload,
    Ping,
    Quality,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Focus::Download => Focus::Upload,
            Focus::Upload => Focus::Ping,
            Focus::Ping => Focus::Quality,
            Focus::Quality => Focus::Download,
        }
    }

    fn prev(self) -> Self {
        match self {
            Focus::Download => Focus::Quality,
            Focus::Upload => Focus::Download,
            Focus::Ping => Focus::Upload,
            Focus::Quality => Focus::Ping,
        }
    }
}

/// Which visualisation fills the left column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    /// Speed over time (`tui-chart` line chart + sparklines).
    Trend,
    /// Speed-tier distribution (`tui-piechart`).
    Distribution,
}

impl View {
    fn toggle(self) -> Self {
        match self {
            View::Trend => View::Distribution,
            View::Distribution => View::Trend,
        }
    }
}

/// Application state for the statistics TUI
pub(crate) struct StatsApp {
    view: View,
    stats: TestStatistics,
    recent_results: Vec<SpeedTestResult>,
    focus: Focus,
    scroll: usize,
    pub(crate) theme: tui_nuances::Theme,
}

impl StatsApp {
    pub(crate) fn new(stats: TestStatistics, recent_results: Vec<SpeedTestResult>) -> Self {
        Self {
            view: View::Trend,
            stats,
            recent_results,
            focus: Focus::Download,
            scroll: 0,
            theme: initial_theme(),
        }
    }

    /// Load the latest stats/results from disk (empty on any error).
    pub(crate) fn from_storage() -> Self {
        let (stats, recent) = load_data().unwrap_or_default();
        Self::new(stats, recent)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.stats.test_count == 0
    }

    /// Handle a navigation key; returns `true` when consumed.
    pub(crate) fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Right | KeyCode::Char('l') => self.focus = self.focus.next(),
            KeyCode::Left | KeyCode::Char('h') => self.focus = self.focus.prev(),
            KeyCode::Char('v') => self.view = self.view.toggle(),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_down(),
            KeyCode::Up | KeyCode::Char('k') => self.scroll_up(),
            _ => return false,
        }
        true
    }

    /// Render the statistics body (cards, charts, table) into `area`.
    pub(crate) fn render_in(&self, frame: &mut ratatui::Frame, area: Rect) {
        PAL.with(|p| p.set(Pal::of(&self.theme)));
        frame.render_widget(
            Block::default().style(Style::default().bg(pal().panel_bg)),
            area,
        );
        render_body(frame, area, self);
    }

    pub(crate) fn latest(&self) -> Option<&SpeedTestResult> {
        self.recent_results.first()
    }

    /// Maximum rows visible in the results table
    fn table_page_size(&self) -> usize {
        8
    }

    fn scroll_down(&mut self) {
        let max = self
            .recent_results
            .len()
            .saturating_sub(self.table_page_size());
        if self.scroll < max {
            self.scroll += 1;
        }
    }

    fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }
}

// ── Public entry-point ────────────────────────────────────────────────────────

/// Launch the interactive statistics TUI.
///
/// Loads data from [`HistoryStorage`], then enters an alternate-screen TUI loop.
/// Returns immediately with an error message printed if no history is found.
pub fn show_statistics_tui() -> io::Result<()> {
    let (stats, recent) = match load_data() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Could not load history: {e}");
            return Ok(());
        }
    };

    if stats.test_count == 0 {
        println!("No test history found. Run a speed test first.");
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = StatsApp::new(stats, recent);
    let result = run_stats_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn load_data() -> Result<(TestStatistics, Vec<SpeedTestResult>), Box<dyn std::error::Error>> {
    let storage = HistoryStorage::new()?;
    let stats = storage.get_statistics()?;
    let recent = storage.get_recent_results(60)?;
    Ok((stats, recent))
}

// ── Event loop ────────────────────────────────────────────────────────────────

fn run_stats_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut StatsApp,
) -> io::Result<()> {
    loop {
        PAL.with(|p| p.set(Pal::of(&app.theme)));
        terminal.draw(|frame| render_stats(frame, app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                        app.focus = app.focus.next();
                    }
                    KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                        app.focus = app.focus.prev();
                    }
                    KeyCode::Char('v') => app.view = app.view.toggle(),
                    KeyCode::Char('t') => app.theme = app.theme.next(),
                    KeyCode::Char('T') => app.theme = app.theme.prev(),
                    KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
                    KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

// ── Rendering ─────────────────────────────────────────────────────────────────

fn render_stats(frame: &mut ratatui::Frame, app: &StatsApp) {
    let area = frame.area();

    // Background fill
    frame.render_widget(
        Block::default().style(Style::default().bg(pal().panel_bg)),
        area,
    );

    // Outer layout: header / body / footer
    let outer = Layout::vertical([
        Constraint::Length(3), // header
        Constraint::Min(0),    // body
        Constraint::Length(3), // footer
    ])
    .split(area);

    render_header(frame, outer[0]);
    render_body(frame, outer[1], app);
    render_footer(frame, outer[2]);
}

fn render_header(frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(pal().cyan))
        .style(Style::default().bg(pal().panel_bg));

    let text = Paragraph::new(Line::from(vec![
        Span::styled("⟨⟨⟨ ", Style::default().fg(pal().cyan)),
        Span::styled(
            "NETRUNNER",
            Style::default()
                .fg(pal().magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" // ", Style::default().fg(pal().dim)),
        Span::styled(
            "STATISTICS DASHBOARD",
            Style::default().fg(pal().cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ⟩⟩⟩", Style::default().fg(pal().cyan)),
    ]))
    .alignment(Alignment::Center)
    .block(block);

    frame.render_widget(text, area);
}

fn render_footer(frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(pal().dim))
        .style(Style::default().bg(pal().panel_bg));

    let text = Line::from(vec![
        Span::styled("Tab/←/→", Style::default().fg(pal().yellow).bold()),
        Span::raw("  Switch chart   "),
        Span::styled("↑/↓", Style::default().fg(pal().yellow).bold()),
        Span::raw("  Scroll results   "),
        Span::styled("t/T", Style::default().fg(pal().yellow).bold()),
        Span::raw("  Theme   "),
        Span::styled("v", Style::default().fg(pal().yellow).bold()),
        Span::raw("  Trend/Distribution   "),
        Span::styled("q / Esc", Style::default().fg(pal().yellow).bold()),
        Span::raw("  Quit"),
    ]);

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(block);

    frame.render_widget(paragraph, area);
}

fn render_body(frame: &mut ratatui::Frame, area: Rect, app: &StatsApp) {
    // Body: left column (charts 2×2) | right column (summary + table)
    let columns =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).split(area);

    render_chart_grid(frame, columns[0], app);
    render_right_panel(frame, columns[1], app);
}

// ── Chart grid ────────────────────────────────────────────────────────────────

fn render_chart_grid(frame: &mut ratatui::Frame, area: Rect, app: &StatsApp) {
    if app.view == View::Trend {
        render_trend(frame, area, app);
        return;
    }
    let rows =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    let top =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[0]);

    let bottom =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(rows[1]);

    render_download_chart(frame, top[0], app);
    render_upload_chart(frame, top[1], app);
    render_ping_chart(frame, bottom[0], app);
    render_quality_chart(frame, bottom[1], app);
}

// ── Trend view (tui-chart) ────────────────────────────────────────────────────

/// Results oldest → newest (storage returns newest first).
fn chronological(app: &StatsApp) -> Vec<&SpeedTestResult> {
    app.recent_results.iter().rev().collect()
}

fn render_trend(frame: &mut ratatui::Frame, area: Rect, app: &StatsApp) {
    let rows =
        Layout::vertical([Constraint::Percentage(58), Constraint::Percentage(42)]).split(area);
    let data = chronological(app);

    let series = |name: &str, color: Color, f: &dyn Fn(&SpeedTestResult) -> f64| {
        Series::new(name)
            .color(color)
            .data(data.iter().enumerate().map(|(i, r)| (i as f64, f(r))))
    };
    let chart = LineChart::new(vec![
        series("Download", pal().green, &|r| r.download_mbps),
        series("Upload", pal().magenta, &|r| r.upload_mbps),
        series("Ping", pal().yellow, &|r| r.ping_ms).secondary_axis(true),
    ])
    .mode(LineMode::Area)
    .legend(ChartLegend::TopLeft)
    .x_axis(Axis::new().title("test #"))
    .y_axis(Axis::new().title("Mbps"))
    .y2_axis(Axis::new().title("ms"))
    .theme(app.theme.clone())
    .block(chart_block("📈 Speed History", true));
    frame.render_widget(&chart, rows[0]);

    let cols = Layout::horizontal([Constraint::Ratio(1, 3); 3]).split(rows[1]);
    let focus = app.focus;
    let spark = |frame: &mut ratatui::Frame,
                 area: Rect,
                 title: &str,
                 unit: &str,
                 color: Color,
                 focused: bool,
                 f: &dyn Fn(&SpeedTestResult) -> f64| {
        let vals: Vec<f64> = data.iter().map(|r| f(r)).collect();
        let last = vals.last().copied().unwrap_or(0.0);
        let max = vals.iter().copied().fold(0.0_f64, f64::max);
        let heading = format!("{title} {last:.1} {unit}");
        let inner = chart_block(&heading, focused);
        let inner_area = inner.inner(area);
        frame.render_widget(inner, area);
        let parts = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).split(inner_area);
        frame.render_widget(
            Sparkline::new(vals)
                .smooth(2)
                .color(color)
                .theme(app.theme.clone()),
            parts[0],
        );
        frame.render_widget(
            Gauge::new(last)
                .range(0.0, max.max(1.0))
                .label(format!("peak {max:.1}"))
                .theme(app.theme.clone()),
            parts[1],
        );
    };
    spark(
        frame,
        cols[0],
        "⬇ Down",
        "Mbps",
        pal().green,
        focus == Focus::Download,
        &|r| r.download_mbps,
    );
    spark(
        frame,
        cols[1],
        "⬆ Up",
        "Mbps",
        pal().magenta,
        focus == Focus::Upload,
        &|r| r.upload_mbps,
    );
    spark(
        frame,
        cols[2],
        "⚡ Ping",
        "ms",
        pal().yellow,
        focus == Focus::Ping,
        &|r| r.ping_ms,
    );
}

/// Build a styled block for a chart cell, highlighted when focused.
fn chart_block<'a>(title: &'a str, focused: bool) -> Block<'a> {
    let (border_color, title_color) = if focused {
        (pal().cyan, pal().cyan)
    } else {
        (pal().dim, Color::Gray)
    };

    Block::default()
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title_style(Style::default().fg(title_color).add_modifier(if focused {
            Modifier::BOLD
        } else {
            Modifier::empty()
        }))
        .padding(Padding::new(1, 1, 0, 0))
        .style(Style::default().bg(pal().panel_bg))
}

// ── Download speed distribution ───────────────────────────────────────────────

fn render_download_chart(frame: &mut ratatui::Frame, area: Rect, app: &StatsApp) {
    let focused = app.focus == Focus::Download;
    // Distribute results into speed tiers
    let (ultra, fast, moderate, slow) =
        bucket_speeds(&app.recent_results, |r| r.download_mbps, 100.0, 50.0, 25.0);

    let slices = build_speed_slices(ultra, fast, moderate, slow);

    let chart = PieChart::new(slices)
        .block(chart_block("⬇  Download Speed", focused))
        .show_legend(true)
        .show_percentages(true)
        .pie_char(if focused {
            symbols::PIE_CHAR_BLOCK
        } else {
            symbols::PIE_CHAR
        })
        .legend_marker(symbols::LEGEND_MARKER_ARROW)
        .legend_position(LegendPosition::Bottom)
        .legend_layout(LegendLayout::Horizontal)
        .legend_alignment(LegendAlignment::Center);

    frame.render_widget(chart, area);
}

// ── Upload speed distribution ─────────────────────────────────────────────────

fn render_upload_chart(frame: &mut ratatui::Frame, area: Rect, app: &StatsApp) {
    let focused = app.focus == Focus::Upload;

    let (ultra, fast, moderate, slow) =
        bucket_speeds(&app.recent_results, |r| r.upload_mbps, 20.0, 10.0, 5.0);

    let slices = build_speed_slices(ultra, fast, moderate, slow);

    let chart = PieChart::new(slices)
        .block(chart_block("⬆  Upload Speed", focused))
        .show_legend(true)
        .show_percentages(true)
        .pie_char(if focused {
            symbols::PIE_CHAR_CIRCLE
        } else {
            symbols::PIE_CHAR
        })
        .legend_marker(symbols::LEGEND_MARKER_CIRCLE)
        .legend_position(LegendPosition::Bottom)
        .legend_layout(LegendLayout::Horizontal)
        .legend_alignment(LegendAlignment::Center);

    frame.render_widget(chart, area);
}

// ── Ping / latency distribution ───────────────────────────────────────────────

fn render_ping_chart(frame: &mut ratatui::Frame, area: Rect, app: &StatsApp) {
    let focused = app.focus == Focus::Ping;

    // Ping buckets: great <20 ms, good 20–50, fair 50–100, poor ≥100
    let mut great = 0u32;
    let mut good = 0u32;
    let mut fair = 0u32;
    let mut poor = 0u32;

    for r in &app.recent_results {
        match r.ping_ms as u64 {
            0..=19 => great += 1,
            20..=49 => good += 1,
            50..=99 => fair += 1,
            _ => poor += 1,
        }
    }

    // Fall back to aggregate stats if no individual results
    if app.recent_results.is_empty() {
        let avg = app.stats.avg_ping_ms;
        match avg as u64 {
            0..=19 => great = 1,
            20..=49 => good = 1,
            50..=99 => fair = 1,
            _ => poor = 1,
        }
    }

    let slices: Vec<PieSlice<'static>> = [
        ("< 20 ms", great as f64, pal().green),
        ("20–50 ms", good as f64, pal().cyan),
        ("50–100 ms", fair as f64, pal().yellow),
        ("> 100 ms", poor as f64, pal().red),
    ]
    .into_iter()
    .filter(|(_, v, _)| *v > 0.0)
    .map(|(label, value, color)| PieSlice::new(label, value, color))
    .collect();

    let slices = if slices.is_empty() {
        vec![PieSlice::new("No data", 1.0, pal().dim)]
    } else {
        slices
    };

    let chart = PieChart::new(slices)
        .block(chart_block("⟳  Ping Latency", focused))
        .show_legend(true)
        .show_percentages(true)
        .pie_char(if focused {
            symbols::PIE_CHAR_DIAMOND
        } else {
            symbols::PIE_CHAR
        })
        .legend_marker(symbols::LEGEND_MARKER_SMALL_CIRCLE)
        .legend_position(LegendPosition::Bottom)
        .legend_layout(LegendLayout::Horizontal)
        .legend_alignment(LegendAlignment::Center);

    frame.render_widget(chart, area);
}

// ── Connection quality distribution ──────────────────────────────────────────

fn render_quality_chart(frame: &mut ratatui::Frame, area: Rect, app: &StatsApp) {
    let focused = app.focus == Focus::Quality;

    let mut excellent = 0u32;
    let mut good = 0u32;
    let mut average = 0u32;
    let mut poor = 0u32;
    let mut failed = 0u32;

    for r in &app.recent_results {
        use netrunner_core::ConnectionQuality;
        match r.quality {
            ConnectionQuality::Excellent => excellent += 1,
            ConnectionQuality::Good => good += 1,
            ConnectionQuality::Average => average += 1,
            ConnectionQuality::Poor | ConnectionQuality::VeryPoor => poor += 1,
            ConnectionQuality::Failed => failed += 1,
        }
    }

    // If no individual results, synthesize from aggregate stats
    if app.recent_results.is_empty() {
        use netrunner_core::ConnectionQuality;
        match ConnectionQuality::from_speed_and_ping(
            app.stats.avg_download_mbps,
            app.stats.avg_upload_mbps,
            app.stats.avg_ping_ms,
        ) {
            ConnectionQuality::Excellent => excellent = 1,
            ConnectionQuality::Good => good = 1,
            ConnectionQuality::Average => average = 1,
            ConnectionQuality::Poor | ConnectionQuality::VeryPoor => poor = 1,
            ConnectionQuality::Failed => failed = 1,
        }
    }

    let slices: Vec<PieSlice<'static>> = [
        ("Excellent", excellent as f64, pal().green),
        ("Good", good as f64, pal().cyan),
        ("Average", average as f64, pal().yellow),
        ("Poor", poor as f64, pal().orange),
        ("Failed", failed as f64, pal().red),
    ]
    .into_iter()
    .filter(|(_, v, _)| *v > 0.0)
    .map(|(label, value, color)| PieSlice::new(label, value, color))
    .collect();

    let slices = if slices.is_empty() {
        vec![PieSlice::new("No data", 1.0, pal().dim)]
    } else {
        slices
    };

    let chart = PieChart::new(slices)
        .block(chart_block("★  Connection Quality", focused))
        .show_legend(true)
        .show_percentages(true)
        .pie_char(if focused {
            symbols::PIE_CHAR_STAR
        } else {
            symbols::PIE_CHAR
        })
        .legend_marker(symbols::LEGEND_MARKER_STAR)
        .legend_position(LegendPosition::Bottom)
        .legend_layout(LegendLayout::Horizontal)
        .legend_alignment(LegendAlignment::Center);

    frame.render_widget(chart, area);
}

// ── Right panel: summary + results table ─────────────────────────────────────

fn render_right_panel(frame: &mut ratatui::Frame, area: Rect, app: &StatsApp) {
    let rows = Layout::vertical([
        Constraint::Length(14), // summary card
        Constraint::Min(0),     // results table
    ])
    .split(area);

    render_summary(frame, rows[0], app);
    render_results_table(frame, rows[1], app);
}

fn render_summary(frame: &mut ratatui::Frame, area: Rect, app: &StatsApp) {
    let s = &app.stats;

    let block = Block::default()
        .title(" 📊  Summary Statistics ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(pal().magenta))
        .title_style(
            Style::default()
                .fg(pal().magenta)
                .add_modifier(Modifier::BOLD),
        )
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(pal().panel_bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let label = |text: &str| -> Span<'static> {
        Span::styled(
            text.to_owned(),
            Style::default().fg(pal().cyan).add_modifier(Modifier::BOLD),
        )
    };
    let value =
        |text: String| -> Span<'static> { Span::styled(text, Style::default().fg(Color::White)) };
    let sep = || -> Span<'static> { Span::styled("  │  ", Style::default().fg(pal().dim)) };

    let test_count_str = if s.test_count == 1 {
        format!("{} test", s.test_count)
    } else {
        format!("{} tests", s.test_count)
    };

    let lines: Vec<Line<'static>> = vec![
        Line::from(vec![label("Tests : "), value(test_count_str)]),
        Line::from(vec![
            label("First : "),
            value(s.first_test.format("%Y-%m-%d %H:%M").to_string()),
        ]),
        Line::from(vec![
            label("Last  : "),
            value(s.last_test.format("%Y-%m-%d %H:%M").to_string()),
        ]),
        Line::from(Span::raw("")),
        Line::from(vec![
            label("⬇ DL avg "),
            value(format!("{:.1} Mbps", s.avg_download_mbps)),
            sep(),
            label("max "),
            value(format!("{:.1}", s.max_download_mbps)),
        ]),
        Line::from(vec![
            label("⬆ UL avg "),
            value(format!("{:.1} Mbps", s.avg_upload_mbps)),
            sep(),
            label("max "),
            value(format!("{:.1}", s.max_upload_mbps)),
        ]),
        Line::from(vec![
            label("⟳ Ping avg"),
            value(format!("{:.1} ms", s.avg_ping_ms)),
            sep(),
            label("min "),
            value(format!("{:.1}", s.min_ping_ms)),
        ]),
        Line::from(Span::raw("")),
        Line::from(vec![
            label("Data ↓ "),
            value(format!("{:.2} GB", s.total_data_downloaded_gb)),
            sep(),
            label("↑ "),
            value(format!("{:.2} GB", s.total_data_uploaded_gb)),
        ]),
    ];

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_results_table(frame: &mut ratatui::Frame, area: Rect, app: &StatsApp) {
    let block = Block::default()
        .title(" 🗂  Recent Results ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(pal().blue))
        .title_style(Style::default().fg(pal().blue).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(pal().panel_bg));

    if app.recent_results.is_empty() {
        let paragraph = Paragraph::new("No results recorded yet.")
            .alignment(Alignment::Center)
            .block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let header_cells = ["Date/Time", "↓ Mbps", "↑ Mbps", "Ping ms", "Quality"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(pal().yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1).bottom_margin(0);

    let page_size = app.table_page_size();
    let start = app.scroll;
    let end = (start + page_size).min(app.recent_results.len());

    let rows: Vec<Row<'_>> = app.recent_results[start..end]
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let quality_color = quality_color(r.quality);
            let row_style = if i % 2 == 0 {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Rgb(180, 180, 200))
            };
            Row::new(vec![
                Cell::from(r.timestamp.format("%m-%d %H:%M").to_string()),
                Cell::from(format!("{:.1}", r.download_mbps)),
                Cell::from(format!("{:.1}", r.upload_mbps)),
                Cell::from(format!("{:.0}", r.ping_ms)),
                Cell::from(format!("{}", r.quality)).style(Style::default().fg(quality_color)),
            ])
            .style(row_style)
        })
        .collect();

    let scroll_hint = format!(
        " 🗂  Recent Results  [{}/{}] ",
        end,
        app.recent_results.len()
    );

    let block_with_hint = Block::default()
        .title(scroll_hint)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(pal().blue))
        .title_style(Style::default().fg(pal().blue).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(pal().panel_bg));

    let widths = [
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Min(9),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block_with_hint)
        .column_spacing(1);

    frame.render_widget(table, area);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Count how many results fall into four speed tiers using `value_fn`.
///
/// - `t1`: ultra threshold (≥ t1)
/// - `t2`: fast  threshold (≥ t2, < t1)
/// - `t3`: moderate threshold (≥ t3, < t2)
/// - below t3 → slow
fn bucket_speeds(
    results: &[SpeedTestResult],
    value_fn: impl Fn(&SpeedTestResult) -> f64,
    t1: f64,
    t2: f64,
    t3: f64,
) -> (u32, u32, u32, u32) {
    let mut ultra = 0u32;
    let mut fast = 0u32;
    let mut moderate = 0u32;
    let mut slow = 0u32;

    for r in results {
        let v = value_fn(r);
        if v >= t1 {
            ultra += 1;
        } else if v >= t2 {
            fast += 1;
        } else if v >= t3 {
            moderate += 1;
        } else {
            slow += 1;
        }
    }

    (ultra, fast, moderate, slow)
}

/// Turn four bucket counts into labelled `PieSlice`s.
/// Slices with zero value are omitted. Falls back to a single "No data" slice.
fn build_speed_slices(ultra: u32, fast: u32, moderate: u32, slow: u32) -> Vec<PieSlice<'static>> {
    let candidates: [(&'static str, f64, Color); 4] = [
        ("Ultra", ultra as f64, pal().green),
        ("Fast", fast as f64, pal().cyan),
        ("Moderate", moderate as f64, pal().yellow),
        ("Slow", slow as f64, pal().red),
    ];

    let slices: Vec<PieSlice<'static>> = candidates
        .into_iter()
        .filter(|(_, v, _)| *v > 0.0)
        .map(|(label, value, color)| PieSlice::new(label, value, color))
        .collect();

    if slices.is_empty() {
        vec![PieSlice::new("No data", 1.0, pal().dim)]
    } else {
        slices
    }
}

// Pick a colour that reflects connection quality. Generated from the canonical
// palette in `netrunner_core` so the TUI and GUI stay in sync.
netrunner_core::quality_color_fn!(fn quality_color -> Color { Color::Rgb });

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    fn app(n: usize) -> StatsApp {
        let results: Vec<SpeedTestResult> = (0..n)
            .map(|i| SpeedTestResult {
                loaded_latency_ms: None,
                download_mbps: 40.0 + i as f64 * 3.0,
                upload_mbps: 10.0 + i as f64,
                ping_ms: 12.0 + (i % 5) as f64,
                ..Default::default()
            })
            .collect();
        let stats = TestStatistics {
            test_count: n,
            ..Default::default()
        };
        StatsApp::new(stats, results)
    }

    #[test]
    fn both_views_render_at_many_sizes() {
        for view in [View::Trend, View::Distribution] {
            for n in [0, 1, 25] {
                for (w, h) in [(140, 45), (100, 30), (60, 20), (20, 8)] {
                    let mut a = app(n);
                    a.view = view;
                    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
                    t.draw(|f| render_stats(f, &a)).unwrap();
                }
            }
        }
    }

    #[test]
    fn trend_view_shows_history_chart() {
        let a = app(12);
        let mut t = Terminal::new(TestBackend::new(140, 45)).unwrap();
        t.draw(|f| render_stats(f, &a)).unwrap();
        assert!(t.backend().to_string().contains("Speed History"));
    }
}
