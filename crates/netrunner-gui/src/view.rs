//! GPUI rendering for the netrunner desktop app.

use gpui::{div, prelude::*, px, rgb, Context, Window};

use crate::app::{Phase, SpeedApp};
use crate::theme::*;

const CHART_HEIGHT: f32 = 170.0;

impl Render for SpeedApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("root")
            .flex()
            .flex_col()
            .size_full()
            .overflow_y_scroll()
            .bg(rgb(BG))
            .text_color(rgb(TEXT))
            .font_family("monospace")
            .p_5()
            .gap_4()
            .child(self.header(cx))
            .child(self.status_bar())
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .w_full()
                    .child(chart_panel(
                        "⬇  DOWNLOAD",
                        self.download_mbps,
                        self.peak_download,
                        &self.download_samples,
                        download_color(),
                        self.phase == Phase::Download,
                    ))
                    .child(chart_panel(
                        "⬆  UPLOAD",
                        self.upload_mbps,
                        self.peak_upload,
                        &self.upload_samples,
                        upload_color(),
                        self.phase == Phase::Upload,
                    )),
            )
            .child(self.summary())
            .child(self.settings_panel(cx))
            .child(self.history_panel(cx))
    }
}

impl SpeedApp {
    fn header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let running = self.running;
        let auto_run = self.settings.auto_run;
        let button_label = if running {
            "RUNNING…"
        } else {
            "▶  RUN TEST"
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .w_full()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(div().text_2xl().text_color(rgb(MAGENTA)).child("⟨⟨⟨"))
                    .child(div().text_2xl().text_color(rgb(CYAN)).child("NETRUNNER"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(MUTED))
                            .child("// SPEED TEST"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    // Auto-run toggle — persisted to settings.json on click.
                    .child(
                        div()
                            .id("autorun")
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .border_1()
                            .border_color(rgb(if auto_run { GREEN } else { PANEL_BORDER }))
                            .text_color(rgb(if auto_run { GREEN } else { MUTED }))
                            .bg(rgb(PANEL_BG))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(PANEL_BORDER)))
                            .on_click(cx.listener(|app, _ev, _window, cx| {
                                app.toggle_auto_run();
                                cx.notify();
                            }))
                            .child(format!("Auto-run: {}", if auto_run { "ON" } else { "OFF" })),
                    )
                    .child(
                        div()
                            .id("run")
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .border_1()
                            .border_color(rgb(if running { MUTED } else { GREEN }))
                            .text_color(rgb(if running { MUTED } else { GREEN }))
                            .bg(rgb(PANEL_BG))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(PANEL_BORDER)))
                            .on_click(cx.listener(|app, _ev, _window, cx| app.start(cx)))
                            .child(button_label),
                    ),
            )
    }

    fn status_bar(&self) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            .w_full()
            .px_3()
            .py_2()
            .rounded_md()
            .bg(rgb(PANEL_BG))
            .border_1()
            .border_color(rgb(PANEL_BORDER))
            .child(
                div()
                    .text_color(rgb(YELLOW))
                    .child(self.phase.label().to_string()),
            )
            .child(div().text_color(rgb(MUTED)).child("·"))
            .child(div().text_color(rgb(TEXT)).child(self.status.to_string()))
    }

    fn summary(&self) -> impl IntoElement {
        let location = self
            .location
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "—".to_string());
        let isp = self
            .isp
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "—".to_string());
        let server = self
            .server
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "—".to_string());

        let quality = self
            .result
            .as_ref()
            .map(|r| r.quality.to_string())
            .unwrap_or_else(|| "—".to_string());
        let quality_col = self
            .result
            .as_ref()
            .map(|r| quality_color(r.quality))
            .unwrap_or_else(|| rgb(MUTED));

        div()
            .flex()
            .flex_col()
            .gap_2()
            .w_full()
            .p_4()
            .rounded_md()
            .bg(rgb(PANEL_BG))
            .border_1()
            .border_color(rgb(MAGENTA))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_6()
                    .child(metric("Ping", format!("{:.0} ms", self.ping_ms), rgb(BLUE)))
                    .child(metric(
                        "Download",
                        format!("{:.1} Mbps", self.download_mbps),
                        rgb(GREEN),
                    ))
                    .child(metric(
                        "Upload",
                        format!("{:.1} Mbps", self.upload_mbps),
                        rgb(CYAN),
                    ))
                    .child(metric("Quality", quality, quality_col)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_6()
                    .text_sm()
                    .child(kv("Location", location))
                    .child(kv("ISP", isp))
                    .child(kv("Server", server)),
            )
    }

    fn settings_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let s = &self.settings;
        let server_host = s
            .server_url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .to_string();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .w_full()
            .p_4()
            .rounded_md()
            .bg(rgb(PANEL_BG))
            .border_1()
            .border_color(rgb(PANEL_BORDER))
            .child(div().text_color(rgb(YELLOW)).child("⚙  SETTINGS"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_end()
                    .gap_6()
                    .child(setting_group(
                        "Server",
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(div().text_color(rgb(TEXT)).child(server_host))
                            .child(ctrl_pill("server-cycle", "↺").on_click(cx.listener(
                                |a, _e, _w, cx| {
                                    a.cycle_server();
                                    cx.notify();
                                },
                            ))),
                    ))
                    .child(setting_group(
                        "Size (MB)",
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(ctrl_pill("size-dec", "−").on_click(cx.listener(
                                |a, _e, _w, cx| {
                                    a.adjust_size(-1);
                                    cx.notify();
                                },
                            )))
                            .child(
                                div()
                                    .w(px(44.))
                                    .text_color(rgb(TEXT))
                                    .child(format!("{}", s.test_size_mb)),
                            )
                            .child(ctrl_pill("size-inc", "+").on_click(cx.listener(
                                |a, _e, _w, cx| {
                                    a.adjust_size(1);
                                    cx.notify();
                                },
                            ))),
                    ))
                    .child(setting_group(
                        "Timeout (s)",
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(ctrl_pill("to-dec", "−").on_click(cx.listener(
                                |a, _e, _w, cx| {
                                    a.adjust_timeout(-5);
                                    cx.notify();
                                },
                            )))
                            .child(
                                div()
                                    .w(px(44.))
                                    .text_color(rgb(TEXT))
                                    .child(format!("{}", s.timeout_seconds)),
                            )
                            .child(ctrl_pill("to-inc", "+").on_click(cx.listener(
                                |a, _e, _w, cx| {
                                    a.adjust_timeout(5);
                                    cx.notify();
                                },
                            ))),
                    ))
                    .child(setting_group(
                        "Detail",
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_color(rgb(TEXT))
                                    .child(s.detail_level.to_string()),
                            )
                            .child(ctrl_pill("detail-cycle", "↺").on_click(cx.listener(
                                |a, _e, _w, cx| {
                                    a.cycle_detail();
                                    cx.notify();
                                },
                            ))),
                    )),
            )
    }

    fn history_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let has_history = !self.history.is_empty();

        let header = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_color(rgb(MAGENTA))
                    .child(format!("🗂  HISTORY ({})", self.history.len())),
            )
            .child(
                div()
                    .id("clear-history")
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .border_1()
                    .border_color(rgb(PANEL_BORDER))
                    .text_color(rgb(MUTED))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(PANEL_BORDER)).text_color(rgb(RED)))
                    .on_click(cx.listener(|app, _ev, _window, cx| {
                        app.clear_history();
                        cx.notify();
                    }))
                    .child("Clear"),
            );

        let body: gpui::AnyElement = if has_history {
            let rows = self.history.iter().map(history_row).collect::<Vec<_>>();
            div()
                .id("history-list")
                .flex()
                .flex_col()
                .gap_1()
                .max_h(px(200.))
                .overflow_y_scroll()
                .children(rows)
                .into_any_element()
        } else {
            div()
                .text_sm()
                .text_color(rgb(MUTED))
                .child("No previous runs yet — run a test to start building history.")
                .into_any_element()
        };

        // Download/upload trend charts across past runs (oldest → newest).
        let charts: gpui::AnyElement = if has_history {
            let downloads: Vec<f32> = self
                .history
                .iter()
                .rev()
                .map(|r| r.download_mbps as f32)
                .collect();
            let uploads: Vec<f32> = self
                .history
                .iter()
                .rev()
                .map(|r| r.upload_mbps as f32)
                .collect();
            div()
                .flex()
                .flex_row()
                .gap_4()
                .w_full()
                .child(history_trend_chart(
                    "⬇ Download trend",
                    &downloads,
                    download_color(),
                ))
                .child(history_trend_chart(
                    "⬆ Upload trend",
                    &uploads,
                    upload_color(),
                ))
                .into_any_element()
        } else {
            div().into_any_element()
        };

        div()
            .flex()
            .flex_col()
            .gap_2()
            .w_full()
            .p_4()
            .rounded_md()
            .bg(rgb(PANEL_BG))
            .border_1()
            .border_color(rgb(BLUE))
            .child(header)
            .child(charts)
            .child(body)
    }
}

/// A single labelled live throughput chart.
fn chart_panel(
    title: &str,
    current: f32,
    peak: f32,
    samples: &[f32],
    color: gpui::Rgba,
    active: bool,
) -> impl IntoElement {
    let max = samples
        .iter()
        .copied()
        .fold(1.0_f32, f32::max)
        .max(peak)
        .max(1.0);

    let border = if active { color } else { rgb(PANEL_BORDER) };

    let bars = samples.iter().map(move |&s| {
        let h = (s / max * CHART_HEIGHT).clamp(2.0, CHART_HEIGHT);
        div().w(px(5.)).h(px(h)).bg(color).rounded_t_sm()
    });

    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap_2()
        .p_4()
        .rounded_md()
        .bg(rgb(PANEL_BG))
        .border_1()
        .border_color(border)
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .child(div().text_color(color).child(title.to_string()))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .child(format!("peak {:.1}", peak)),
                ),
        )
        .child(
            div()
                .text_2xl()
                .text_color(rgb(TEXT))
                .child(format!("{:.1} Mbps", current)),
        )
        .child(
            div()
                .flex()
                .flex_row()
                .items_end()
                .gap(px(1.))
                .h(px(CHART_HEIGHT))
                .w_full()
                .overflow_hidden()
                .children(bars),
        )
}

fn metric(label: &str, value: String, color: gpui::Rgba) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_color(rgb(MUTED))
                .child(label.to_string()),
        )
        .child(div().text_xl().text_color(color).child(value))
}

fn kv(label: &str, value: String) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .gap_2()
        .child(div().text_color(rgb(MUTED)).child(format!("{label}:")))
        .child(div().text_color(rgb(TEXT)).child(value))
}

/// A small clickable control pill (stepper / cycle button). Attach `.on_click`.
fn ctrl_pill(id: &'static str, label: impl Into<String>) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .px_2()
        .py_1()
        .rounded_md()
        .border_1()
        .border_color(rgb(PANEL_BORDER))
        .text_color(rgb(CYAN))
        .cursor_pointer()
        .hover(|s| s.bg(rgb(PANEL_BORDER)))
        .child(label.into())
}

/// A labelled settings control group (small caption above its controls).
fn setting_group(name: &str, controls: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .text_xs()
                .text_color(rgb(MUTED))
                .child(name.to_string()),
        )
        .child(controls)
}

/// A compact bar chart of a metric across past runs (oldest → newest).
fn history_trend_chart(title: &str, values: &[f32], color: gpui::Rgba) -> impl IntoElement {
    const H: f32 = 64.0;
    let max = values.iter().copied().fold(1.0_f32, f32::max).max(1.0);
    let latest = values.last().copied().unwrap_or(0.0);
    let bars = values.iter().map(move |&v| {
        let bh = (v / max * H).clamp(2.0, H);
        div().w(px(7.)).h(px(bh)).bg(color).rounded_t_sm()
    });

    div()
        .flex()
        .flex_col()
        .flex_1()
        .gap_1()
        .child(
            div()
                .flex()
                .flex_row()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .child(title.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(color)
                        .child(format!("{latest:.1} Mbps")),
                ),
        )
        .child(
            div()
                .id(gpui::ElementId::Name(title.to_string().into()))
                .flex()
                .flex_row()
                .items_end()
                .gap(px(2.))
                .h(px(H))
                .w_full()
                .overflow_x_scroll()
                .children(bars),
        )
}

/// A single row in the history list: date, download, upload, ping, quality.
fn history_row(r: &netrunner_core::SpeedTestResult) -> impl IntoElement {
    let when = r.timestamp.format("%m-%d %H:%M").to_string();
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_4()
        .w_full()
        .px_2()
        .py_1()
        .rounded_sm()
        .hover(|s| s.bg(rgb(PANEL_BORDER)))
        .child(
            div()
                .w(px(96.))
                .text_sm()
                .text_color(rgb(MUTED))
                .child(when),
        )
        .child(
            div()
                .w(px(92.))
                .text_color(rgb(GREEN))
                .child(format!("↓ {:.1}", r.download_mbps)),
        )
        .child(
            div()
                .w(px(92.))
                .text_color(rgb(CYAN))
                .child(format!("↑ {:.1}", r.upload_mbps)),
        )
        .child(
            div()
                .w(px(72.))
                .text_color(rgb(BLUE))
                .child(format!("{:.0} ms", r.ping_ms)),
        )
        .child(
            div()
                .text_color(quality_color(r.quality))
                .child(r.quality.to_string()),
        )
}
