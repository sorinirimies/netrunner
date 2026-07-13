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
use tui_piechart::{symbols, LegendAlignment, LegendLayout, LegendPosition, PieChart, PieSlice};

use netrunner_core::{HistoryStorage, SpeedTestResult, TestStatistics};

// ── Cyberpunk colour palette ─────────────────────────────────────────────────
const COLOR_CYAN: Color = Color::Rgb(0, 255, 255);
const COLOR_MAGENTA: Color = Color::Rgb(255, 0, 255);
const COLOR_GREEN: Color = Color::Rgb(0, 255, 128);
const COLOR_YELLOW: Color = Color::Rgb(255, 220, 0);
const COLOR_ORANGE: Color = Color::Rgb(255, 140, 0);
const COLOR_RED: Color = Color::Rgb(255, 60, 60);
const COLOR_BLUE: Color = Color::Rgb(60, 140, 255);

const COLOR_DIM: Color = Color::Rgb(80, 80, 100);
const COLOR_PANEL_BG: Color = Color::Rgb(10, 10, 20);

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

/// Application state for the statistics TUI
struct StatsApp {
    stats: TestStatistics,
    recent_results: Vec<SpeedTestResult>,
    focus: Focus,
    scroll: usize,
}

impl StatsApp {
    fn new(stats: TestStatistics, recent_results: Vec<SpeedTestResult>) -> Self {
        Self {
            stats,
            recent_results,
            focus: Focus::Download,
            scroll: 0,
        }
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
    let recent = storage.get_recent_results(20)?;
    Ok((stats, recent))
}

// ── Event loop ────────────────────────────────────────────────────────────────

fn run_stats_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut StatsApp,
) -> io::Result<()> {
    loop {
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
        Block::default().style(Style::default().bg(COLOR_PANEL_BG)),
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
        .border_style(Style::default().fg(COLOR_CYAN))
        .style(Style::default().bg(COLOR_PANEL_BG));

    let text = Paragraph::new(Line::from(vec![
        Span::styled("⟨⟨⟨ ", Style::default().fg(COLOR_CYAN)),
        Span::styled(
            "NETRUNNER",
            Style::default()
                .fg(COLOR_MAGENTA)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" // ", Style::default().fg(COLOR_DIM)),
        Span::styled(
            "STATISTICS DASHBOARD",
            Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ⟩⟩⟩", Style::default().fg(COLOR_CYAN)),
    ]))
    .alignment(Alignment::Center)
    .block(block);

    frame.render_widget(text, area);
}

fn render_footer(frame: &mut ratatui::Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_DIM))
        .style(Style::default().bg(COLOR_PANEL_BG));

    let text = Line::from(vec![
        Span::styled("Tab/←/→", Style::default().fg(COLOR_YELLOW).bold()),
        Span::raw("  Switch chart   "),
        Span::styled("↑/↓", Style::default().fg(COLOR_YELLOW).bold()),
        Span::raw("  Scroll results   "),
        Span::styled("q / Esc", Style::default().fg(COLOR_YELLOW).bold()),
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

/// Build a styled block for a chart cell, highlighted when focused.
fn chart_block<'a>(title: &'a str, focused: bool) -> Block<'a> {
    let (border_color, title_color) = if focused {
        (COLOR_CYAN, COLOR_CYAN)
    } else {
        (COLOR_DIM, Color::Gray)
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
        .style(Style::default().bg(COLOR_PANEL_BG))
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
        ("< 20 ms", great as f64, COLOR_GREEN),
        ("20–50 ms", good as f64, COLOR_CYAN),
        ("50–100 ms", fair as f64, COLOR_YELLOW),
        ("> 100 ms", poor as f64, COLOR_RED),
    ]
    .into_iter()
    .filter(|(_, v, _)| *v > 0.0)
    .map(|(label, value, color)| PieSlice::new(label, value, color))
    .collect();

    let slices = if slices.is_empty() {
        vec![PieSlice::new("No data", 1.0, COLOR_DIM)]
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
        ("Excellent", excellent as f64, COLOR_GREEN),
        ("Good", good as f64, COLOR_CYAN),
        ("Average", average as f64, COLOR_YELLOW),
        ("Poor", poor as f64, COLOR_ORANGE),
        ("Failed", failed as f64, COLOR_RED),
    ]
    .into_iter()
    .filter(|(_, v, _)| *v > 0.0)
    .map(|(label, value, color)| PieSlice::new(label, value, color))
    .collect();

    let slices = if slices.is_empty() {
        vec![PieSlice::new("No data", 1.0, COLOR_DIM)]
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
        .border_style(Style::default().fg(COLOR_MAGENTA))
        .title_style(
            Style::default()
                .fg(COLOR_MAGENTA)
                .add_modifier(Modifier::BOLD),
        )
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(COLOR_PANEL_BG));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let label = |text: &str| -> Span<'static> {
        Span::styled(
            text.to_owned(),
            Style::default().fg(COLOR_CYAN).add_modifier(Modifier::BOLD),
        )
    };
    let value =
        |text: String| -> Span<'static> { Span::styled(text, Style::default().fg(Color::White)) };
    let sep = || -> Span<'static> { Span::styled("  │  ", Style::default().fg(COLOR_DIM)) };

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
        .border_style(Style::default().fg(COLOR_BLUE))
        .title_style(Style::default().fg(COLOR_BLUE).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(COLOR_PANEL_BG));

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
                    .fg(COLOR_YELLOW)
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
        .border_style(Style::default().fg(COLOR_BLUE))
        .title_style(Style::default().fg(COLOR_BLUE).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(COLOR_PANEL_BG));

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
        ("Ultra", ultra as f64, COLOR_GREEN),
        ("Fast", fast as f64, COLOR_CYAN),
        ("Moderate", moderate as f64, COLOR_YELLOW),
        ("Slow", slow as f64, COLOR_RED),
    ];

    let slices: Vec<PieSlice<'static>> = candidates
        .into_iter()
        .filter(|(_, v, _)| *v > 0.0)
        .map(|(label, value, color)| PieSlice::new(label, value, color))
        .collect();

    if slices.is_empty() {
        vec![PieSlice::new("No data", 1.0, COLOR_DIM)]
    } else {
        slices
    }
}

// Pick a colour that reflects connection quality. Generated from the canonical
// palette in `netrunner_core` so the TUI and GUI stay in sync.
netrunner_core::quality_color_fn!(fn quality_color -> Color { Color::Rgb });
