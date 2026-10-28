//! The Netrunner workbench: a persistent, tabbed ratatui application.
//!
//! Unlike the one-shot live screen, this stays open so you can run test after
//! test, browse history, import/export it and tweak settings:
//!
//! | Tab      | What it does                                                        |
//! |----------|---------------------------------------------------------------------|
//! | Test     | live `tui-chart` dashboard (`tui-spinner`), latest result (`tui-rating`) |
//! | History  | speed-over-time chart, sparklines, `tui-piechart` distribution      |
//! | Files    | `tui-file-explorer` to import a history `.json`                     |
//! | Settings | `tui-checkbox` / `tui-slider` controls, persisted via `Settings`    |

use std::future::Future;
use std::io;
use std::pin::Pin;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use netrunner_core::{
    ConnectionQuality, HistoryStorage, Settings, SpeedTest, SpeedTestResult, TestEvent,
};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Tabs},
    Frame,
};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tui_checkbox::Checkbox;
use tui_file_explorer::{render_themed, ExplorerOutcome, FileExplorer};
use tui_nuances::Theme;
use tui_rating::Rating;
use tui_slider::Slider;
use tui_spinner::FluxSpinner;

use crate::live::{draw_dashboard, LiveState};
use crate::stats_ui::StatsApp;

type Engine = Pin<Box<dyn Future<Output = Result<SpeedTestResult, String>>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Test,
    History,
    Files,
    Settings,
}

const TABS: [(Tab, &str); 4] = [
    (Tab::Test, "⚡ Test"),
    (Tab::History, "📈 History"),
    (Tab::Files, "📁 Files"),
    (Tab::Settings, "⚙ Settings"),
];

/// Side effects the event loop performs on behalf of key handling.
#[derive(Debug, PartialEq, Eq)]
enum Action {
    StartTest,
    CancelTest,
    Quit,
}

/// One row of the settings tab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    Animations,
    AutoRun,
    Timeout,
    HistorySize,
}

const ROWS: [Row; 4] = [
    Row::Animations,
    Row::AutoRun,
    Row::Timeout,
    Row::HistorySize,
];

pub struct Workbench {
    tab: Tab,
    theme: Theme,
    live: LiveState,
    running: bool,
    stats: StatsApp,
    explorer: FileExplorer,
    settings: Settings,
    row: usize,
    status: String,
    help: bool,
}

impl Workbench {
    pub fn new(theme: Theme) -> Self {
        let dir = std::env::current_dir().unwrap_or_else(|_| ".".into());
        let mut stats = StatsApp::from_storage();
        stats.theme = theme.clone();
        Self {
            tab: Tab::Test,
            live: LiveState::new(theme.clone()),
            theme,
            running: false,
            stats,
            explorer: FileExplorer::new(dir, vec!["json".into()]),
            settings: Settings::load(),
            row: 0,
            status: "Ready — press Enter to run a speed test".into(),
            help: false,
        }
    }

    fn set_theme(&mut self, theme: Theme) {
        self.live.theme = theme.clone();
        self.stats.theme = theme.clone();
        self.theme = theme;
    }

    fn refresh_history(&mut self) {
        self.stats = StatsApp::from_storage();
        self.stats.theme = self.theme.clone();
    }

    // ── Input ────────────────────────────────────────────────────────────────

    fn on_key(&mut self, key: crossterm::event::KeyEvent) -> Option<Action> {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Some(Action::Quit);
        }
        if self.help {
            self.help = false;
            return None;
        }
        // The explorer owns the keyboard while searching (so `q`, `t` … type).
        let capturing = self.tab == Tab::Files && self.explorer.is_searching();

        if !capturing {
            match key.code {
                KeyCode::F(n @ 1..=4) => {
                    self.tab = TABS[n as usize - 1].0;
                    return None;
                }
                KeyCode::Char('?') => {
                    self.help = true;
                    return None;
                }
                KeyCode::Char('t') if self.tab != Tab::Files => {
                    self.set_theme(self.theme.next());
                    return None;
                }
                KeyCode::Char('T') if self.tab != Tab::Files => {
                    self.set_theme(self.theme.prev());
                    return None;
                }
                _ => {}
            }
            if self.tab != Tab::Files {
                match key.code {
                    KeyCode::Char('q') => return Some(Action::Quit),
                    KeyCode::Char(c @ '1'..='4') => {
                        self.tab = TABS[c as usize - '1' as usize].0;
                        return None;
                    }
                    _ => {}
                }
            }
            match key.code {
                KeyCode::Tab => {
                    self.cycle_tab(1);
                    return None;
                }
                KeyCode::BackTab => {
                    self.cycle_tab(-1);
                    return None;
                }
                _ => {}
            }
        }

        match self.tab {
            Tab::Test => self.key_test(key.code),
            Tab::History => {
                self.key_history(key.code);
                None
            }
            Tab::Files => {
                self.key_files(key);
                None
            }
            Tab::Settings => {
                self.key_settings(key.code);
                None
            }
        }
    }

    fn cycle_tab(&mut self, delta: isize) {
        let i = TABS.iter().position(|(t, _)| *t == self.tab).unwrap_or(0) as isize;
        let n = TABS.len() as isize;
        self.tab = TABS[((i + delta).rem_euclid(n)) as usize].0;
    }

    fn key_test(&mut self, code: KeyCode) -> Option<Action> {
        match code {
            KeyCode::Enter | KeyCode::Char('r') | KeyCode::Char(' ') if !self.running => {
                Some(Action::StartTest)
            }
            KeyCode::Esc | KeyCode::Char('c') if self.running => Some(Action::CancelTest),
            _ => None,
        }
    }

    fn key_history(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('e') => self.export_history(),
            KeyCode::Char('i') => self.tab = Tab::Files,
            KeyCode::Esc => self.tab = Tab::Test,
            other => {
                self.stats.handle_key(other);
            }
        }
    }

    fn export_history(&mut self) {
        let path = self
            .explorer
            .current_dir
            .join(format!("netrunner-history-{}.json", chrono_stamp()));
        let result = HistoryStorage::new().and_then(|s| s.export_to_json(&path.to_string_lossy()));
        self.status = match result {
            Ok(()) => format!("Exported history → {}", path.display()),
            Err(e) => format!("Export failed: {e}"),
        };
    }

    fn key_files(&mut self, key: crossterm::event::KeyEvent) {
        if let KeyCode::F(_) = key.code {
            return;
        }
        match self.explorer.handle_key(key) {
            ExplorerOutcome::Selected(path) => self.import_history(&path),
            ExplorerOutcome::Dismissed => self.tab = Tab::History,
            _ => {}
        }
    }

    fn import_history(&mut self, path: &std::path::Path) {
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            self.status = "Pick a history .json file to import".into();
            return;
        }
        let result =
            HistoryStorage::new().and_then(|s| s.import_from_json(&path.to_string_lossy()));
        match result {
            Ok(n) => {
                self.refresh_history();
                self.status = format!("Imported {n} results from {}", path.display());
                self.tab = Tab::History;
            }
            Err(e) => self.status = format!("Import failed: {e}"),
        }
    }

    fn key_settings(&mut self, code: KeyCode) {
        let n = ROWS.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => self.row = (self.row + n - 1) % n,
            KeyCode::Down | KeyCode::Char('j') => self.row = (self.row + 1) % n,
            KeyCode::Left | KeyCode::Char('h') => self.adjust(-1),
            KeyCode::Right | KeyCode::Char('l') => self.adjust(1),
            KeyCode::Char(' ') | KeyCode::Enter => self.adjust(1),
            KeyCode::Esc => self.tab = Tab::Test,
            _ => {}
        }
    }

    fn adjust(&mut self, dir: i64) {
        let s = &mut self.settings;
        match ROWS[self.row] {
            Row::Animations => s.animations = !s.animations,
            Row::AutoRun => s.auto_run = !s.auto_run,
            Row::Timeout => s.timeout_seconds = step(s.timeout_seconds, dir * 5, 5, 120),
            Row::HistorySize => {
                s.max_history = step(s.max_history as u64, dir * 10, 10, 500) as usize
            }
        }
        self.status = match self.settings.save() {
            Ok(()) => "Settings saved".into(),
            Err(e) => format!("Could not save settings: {e}"),
        };
    }

    // ── Test lifecycle ──────────────────────────────────────────────────────

    fn start_test(&mut self) -> Option<(Engine, UnboundedReceiver<TestEvent>)> {
        let (tx, rx) = mpsc::unbounded_channel();
        match SpeedTest::with_events(self.settings.to_config(), Some(tx)) {
            Ok(test) => {
                let tick = self.live.tick;
                self.live = LiveState::new(self.theme.clone());
                self.live.tick = tick;
                self.running = true;
                self.tab = Tab::Test;
                self.status = "Running speed test…".into();
                let engine: Engine =
                    Box::pin(async move { test.run_full_test().await.map_err(|e| e.to_string()) });
                Some((engine, rx))
            }
            Err(e) => {
                self.status = format!("Could not start test: {e}");
                None
            }
        }
    }

    fn finish_test(&mut self, result: Result<SpeedTestResult, String>) {
        self.running = false;
        self.live.finished = true;
        match result {
            Ok(r) => {
                self.live.done = true;
                let saved = if r.quality == ConnectionQuality::Failed {
                    // Don't pollute history with failed runs (no throughput at all).
                    Err("run failed, not saved".into())
                } else {
                    HistoryStorage::new().and_then(|s| s.save_result(&r))
                };
                self.refresh_history();
                self.status = match saved {
                    Ok(()) => format!(
                        "Done — ⬇ {:.1} ⬆ {:.1} Mbps · {:.0} ms · saved to history",
                        r.download_mbps, r.upload_mbps, r.ping_ms
                    ),
                    Err(e) => match &self.live.warning {
                        Some(w) => format!("⚠ {w}"),
                        None => format!("Not saved to history: {e}"),
                    },
                };
            }
            Err(e) => self.status = format!("Test failed: {e}"),
        }
    }

    // ── Rendering ───────────────────────────────────────────────────────────

    pub fn draw(&mut self, frame: &mut Frame) {
        let t = self.theme.clone();
        let area = frame.area();
        frame.render_widget(
            Block::default()
                .style(Style::default().bg(t.background.unwrap_or(ratatui::style::Color::Reset))),
            area,
        );
        let [head, body, foot] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .areas(area);

        self.draw_head(frame, head);
        match self.tab {
            Tab::Test => self.draw_test(frame, body),
            Tab::History => self.draw_history(frame, body),
            Tab::Files => {
                let fe_theme = tui_file_explorer::Theme::from(&t);
                render_themed(&mut self.explorer, frame, body, &fe_theme);
            }
            Tab::Settings => self.draw_settings(frame, body),
        }
        self.draw_foot(frame, foot);
        if self.help {
            self.draw_help(frame, area);
        }
    }

    fn draw_head(&self, frame: &mut Frame, area: Rect) {
        let t = &self.theme;
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(t.border))
            .title(Span::styled(
                " NETRUNNER ",
                Style::default().fg(t.accent).add_modifier(Modifier::BOLD),
            ));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let [tabs, spin] =
            Layout::horizontal([Constraint::Min(0), Constraint::Length(14)]).areas(inner);
        let idx = TABS.iter().position(|(x, _)| *x == self.tab).unwrap_or(0);
        frame.render_widget(
            Tabs::new(TABS.iter().map(|(_, n)| Line::from(*n)).collect::<Vec<_>>())
                .select(idx)
                .style(Style::default().fg(t.muted))
                .highlight_style(
                    Style::default()
                        .fg(t.accent)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )
                .divider("│"),
            tabs,
        );
        if self.running {
            let [s, label] =
                Layout::horizontal([Constraint::Length(2), Constraint::Min(0)]).areas(spin);
            frame.render_widget(FluxSpinner::new(self.live.tick / 2).color(t.accent), s);
            frame.render_widget(
                Paragraph::new("testing").style(Style::default().fg(t.accent)),
                label,
            );
        } else {
            frame.render_widget(
                Paragraph::new(format!("◐ {}", t.name))
                    .alignment(Alignment::Right)
                    .style(Style::default().fg(t.muted)),
                spin,
            );
        }
    }

    fn draw_foot(&self, frame: &mut Frame, area: Rect) {
        let t = &self.theme;
        let key = |k: &'static str| Span::styled(k, Style::default().fg(t.warning).bold());
        let mut hints = vec![key("Tab"), Span::raw(" switch  ")];
        match self.tab {
            Tab::Test if self.running => hints.extend([key("Esc"), Span::raw(" cancel  ")]),
            Tab::Test => hints.extend([key("Enter"), Span::raw(" run  ")]),
            Tab::History => hints.extend([
                key("v"),
                Span::raw(" chart/pies  "),
                key("e"),
                Span::raw(" export  "),
                key("i"),
                Span::raw(" import  "),
            ]),
            Tab::Files => hints.extend([
                key("Enter"),
                Span::raw(" import .json  "),
                key("Esc"),
                Span::raw(" back  "),
            ]),
            Tab::Settings => hints.extend([
                key("↑↓"),
                Span::raw(" select  "),
                key("←→"),
                Span::raw(" change  "),
            ]),
        }
        hints.extend([
            key("t"),
            Span::raw(" theme  "),
            key("?"),
            Span::raw(" help  "),
            key("q"),
            Span::raw(" quit"),
        ]);
        let lines = vec![
            Line::styled(self.status.clone(), Style::default().fg(t.foreground)),
            Line::from(hints),
        ];
        frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
    }

    fn draw_test(&self, frame: &mut Frame, area: Rect) {
        let showing_run = self.running || self.live.done || self.live.finished;
        if showing_run {
            draw_dashboard(frame, area, &self.live);
            return;
        }
        let t = &self.theme;
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(t.border))
            .title(Span::styled(
                " LATEST RESULT ",
                Style::default().fg(t.accent).bold(),
            ));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let [top, mid, bottom] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .flex(ratatui::layout::Flex::Center)
        .areas(inner);
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("Press ", Style::default().fg(t.muted)),
                Span::styled("Enter", Style::default().fg(t.warning).bold()),
                Span::styled(" to run a speed test", Style::default().fg(t.muted)),
            ]))
            .alignment(Alignment::Center),
            top,
        );
        match self.stats.latest() {
            Some(r) => {
                frame.render_widget(
                    Rating::new(stars(r.quality), 5)
                        .alignment(Alignment::Center)
                        .theme(t.clone()),
                    mid,
                );
                let mut lines = vec![
                    Line::from(vec![
                        Span::styled(
                            format!("⬇ {:.1} Mbps", r.download_mbps),
                            Style::default().fg(t.success).bold(),
                        ),
                        Span::raw("    "),
                        Span::styled(
                            format!("⬆ {:.1} Mbps", r.upload_mbps),
                            Style::default().fg(t.secondary).bold(),
                        ),
                        Span::raw("    "),
                        Span::styled(
                            format!("⚡ {:.0} ms", r.ping_ms),
                            Style::default().fg(t.warning).bold(),
                        ),
                    ]),
                    Line::styled(
                        format!(
                            "{:?} · jitter {:.1} ms · loss {:.1}%{} · {}",
                            r.quality,
                            r.jitter_ms,
                            r.packet_loss_percent,
                            r.bufferbloat_grade()
                                .map_or(String::new(), |g| format!(" · bufferbloat {g}")),
                            r.timestamp.format("%Y-%m-%d %H:%M")
                        ),
                        Style::default().fg(t.muted),
                    ),
                ];
                if let Some(isp) = &r.isp {
                    lines.push(Line::styled(isp.clone(), Style::default().fg(t.muted)));
                }
                frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), bottom);
            }
            None => frame.render_widget(
                Paragraph::new("No results yet.")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(t.muted)),
                mid,
            ),
        }
    }

    fn draw_history(&self, frame: &mut Frame, area: Rect) {
        if self.stats.is_empty() {
            let t = &self.theme;
            frame.render_widget(
                Paragraph::new("No history yet — run a test (1) or import a file (i).")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(t.muted)),
                area,
            );
        } else {
            self.stats.render_in(frame, area);
        }
    }

    fn draw_settings(&self, frame: &mut Frame, area: Rect) {
        let t = &self.theme;
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(t.border))
            .title(Span::styled(
                " SETTINGS ",
                Style::default().fg(t.accent).bold(),
            ));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let rows = Layout::vertical(vec![Constraint::Length(2); ROWS.len() + 1]).split(inner);
        for (i, row) in ROWS.iter().enumerate() {
            let focused = i == self.row;
            let marker = if focused { "▶ " } else { "  " };
            let cell =
                Layout::horizontal([Constraint::Length(2), Constraint::Min(0)]).split(rows[i]);
            frame.render_widget(
                Paragraph::new(marker).style(Style::default().fg(t.accent)),
                cell[0],
            );
            match row {
                Row::Animations => frame.render_widget(
                    Checkbox::new("Live dashboard & animations", self.settings.animations)
                        .theme(t.clone()),
                    cell[1],
                ),
                Row::AutoRun => frame.render_widget(
                    Checkbox::new("Run a test on launch", self.settings.auto_run).theme(t.clone()),
                    cell[1],
                ),
                Row::Timeout => frame.render_widget(
                    Slider::new(self.settings.timeout_seconds as f64, 5.0, 120.0)
                        .label("Timeout (s)")
                        .show_value(true)
                        .theme(t),
                    cell[1],
                ),
                Row::HistorySize => frame.render_widget(
                    Slider::new(self.settings.max_history as f64, 10.0, 500.0)
                        .label("History size")
                        .show_value(true)
                        .theme(t),
                    cell[1],
                ),
            }
        }
        frame.render_widget(
            Paragraph::new(format!(
                "Theme: {} (t/T) · server: {} · saved to {}",
                t.name,
                self.settings.server_url,
                Settings::config_path().map_or("n/a".into(), |p| p.display().to_string()),
            ))
            .style(Style::default().fg(t.muted)),
            rows[ROWS.len()],
        );
    }

    fn draw_help(&self, frame: &mut Frame, area: Rect) {
        let t = &self.theme;
        let w = 54.min(area.width);
        let h = 14.min(area.height);
        let r = Rect::new(
            area.x + (area.width - w) / 2,
            area.y + (area.height - h) / 2,
            w,
            h,
        );
        frame.render_widget(Clear, r);
        let text = [
            "Global",
            "  Tab / Shift-Tab / 1-4 / F1-F4   switch tab",
            "  t / T                           cycle theme",
            "  q / Ctrl-C                      quit",
            "Test      Enter run · Esc cancel",
            "History   v chart/pies · ←→ focus · ↑↓ scroll",
            "          e export · i import",
            "Files     tui-file-explorer · Enter imports .json",
            "Settings  ↑↓ select · ←→/Space change (auto-saves)",
            "",
            "Press any key to close",
        ];
        frame.render_widget(
            Paragraph::new(text.join("\n"))
                .style(Style::default().fg(t.foreground))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Double)
                        .border_style(Style::default().fg(t.accent))
                        .title(" Help "),
                ),
            r,
        );
    }
}

fn stars(q: ConnectionQuality) -> f64 {
    match q {
        ConnectionQuality::Excellent => 5.0,
        ConnectionQuality::Good => 4.0,
        ConnectionQuality::Average => 3.0,
        ConnectionQuality::Poor => 2.0,
        ConnectionQuality::VeryPoor => 1.0,
        _ => 0.0,
    }
}

fn step(v: u64, delta: i64, min: u64, max: u64) -> u64 {
    (v as i64 + delta).clamp(min as i64, max as i64) as u64
}

fn chrono_stamp() -> String {
    chrono::Local::now().format("%Y%m%d-%H%M%S").to_string()
}

fn initial_theme() -> Theme {
    std::env::var("NETRUNNER_THEME")
        .ok()
        .and_then(|n| Theme::by_name(&n))
        .or_else(|| Theme::by_name("cyberpunk"))
        .unwrap_or_default()
}

struct TerminalGuard;
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

/// Run the workbench until the user quits.
pub async fn run_workbench() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let _guard = TerminalGuard;
    let mut wb = Workbench::new(initial_theme());
    let mut frames = tokio::time::interval(Duration::from_millis(50));
    let mut engine: Option<Engine> = None;
    let mut rx: Option<UnboundedReceiver<TestEvent>> = None;
    let mut pending = if wb.settings.auto_run {
        Some(Action::StartTest)
    } else {
        None
    };

    loop {
        if let Some(action) = pending.take() {
            match action {
                Action::Quit => break,
                Action::CancelTest => {
                    engine = None;
                    rx = None;
                    wb.running = false;
                    wb.live = LiveState::new(wb.theme.clone());
                    wb.status = "Test cancelled".into();
                }
                Action::StartTest => {
                    if let Some((e, r)) = wb.start_test() {
                        engine = Some(e);
                        rx = Some(r);
                    }
                }
            }
        }

        tokio::select! {
            biased;
            ev = async {
                match rx.as_mut() {
                    Some(r) => r.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                if let Some(ev) = ev { wb.live.apply(ev); }
            }
            res = async {
                match engine.as_mut() {
                    Some(e) => e.as_mut().await,
                    None => std::future::pending().await,
                }
            } => {
                engine = None;
                if let Some(r) = rx.as_mut() {
                    while let Ok(ev) = r.try_recv() { wb.live.apply(ev); }
                }
                rx = None;
                wb.finish_test(res);
            }
            _ = frames.tick() => {
                wb.live.tick += 1;
                terminal.draw(|f| wb.draw(f))?;
                while event::poll(Duration::ZERO)? {
                    if let Event::Key(k) = event::read()? {
                        if k.kind == KeyEventKind::Press && pending.is_none() {
                            pending = wb.on_key(k);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyEventState};
    use ratatui::{backend::TestBackend, Terminal};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn bench() -> Workbench {
        let mut wb = Workbench::new(Theme::default());
        wb.settings = Settings::default();
        wb
    }

    #[test]
    fn renders_every_tab_at_many_sizes() {
        let mut wb = bench();
        for tab in [Tab::Test, Tab::History, Tab::Files, Tab::Settings] {
            wb.tab = tab;
            for (w, h) in [(140, 45), (100, 30), (60, 20), (20, 8)] {
                let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
                t.draw(|f| wb.draw(f)).unwrap();
            }
        }
        wb.help = true;
        let mut t = Terminal::new(TestBackend::new(100, 30)).unwrap();
        t.draw(|f| wb.draw(f)).unwrap();
    }

    #[test]
    fn tab_navigation_and_actions() {
        let mut wb = bench();
        assert_eq!(wb.on_key(key(KeyCode::Tab)), None);
        assert_eq!(wb.tab, Tab::History);
        assert_eq!(wb.on_key(key(KeyCode::Char('4'))), None);
        assert_eq!(wb.tab, Tab::Settings);
        assert_eq!(wb.on_key(key(KeyCode::BackTab)), None);
        assert_eq!(wb.tab, Tab::Files);
        // Tab still switches while in the explorer (it does not use it).
        assert_eq!(wb.on_key(key(KeyCode::Tab)), None);
        assert_eq!(wb.tab, Tab::Settings);
        assert_eq!(wb.on_key(key(KeyCode::BackTab)), None);
        // F-keys work even inside the explorer.
        assert_eq!(wb.on_key(key(KeyCode::F(1))), None);
        assert_eq!(wb.tab, Tab::Test);
        assert_eq!(wb.on_key(key(KeyCode::Enter)), Some(Action::StartTest));
        assert_eq!(wb.on_key(key(KeyCode::Char('q'))), Some(Action::Quit));
        wb.running = true;
        assert_eq!(wb.on_key(key(KeyCode::Esc)), Some(Action::CancelTest));
        assert_eq!(wb.on_key(key(KeyCode::Enter)), None);
    }

    #[test]
    fn step_clamps() {
        assert_eq!(step(30, 5, 5, 120), 35);
        assert_eq!(step(5, -5, 5, 120), 5);
        assert_eq!(step(118, 5, 5, 120), 120);
    }

    #[test]
    fn stars_map_quality() {
        assert_eq!(stars(ConnectionQuality::Excellent), 5.0);
        assert_eq!(stars(ConnectionQuality::Failed), 0.0);
    }
}
