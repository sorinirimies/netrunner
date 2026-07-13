//! GPUI rendering for the netrunner desktop app.

use gpui::{div, prelude::*, px, rgb, Context, Window};

use crate::app::{Phase, SpeedApp};
use crate::theme::*;

const CHART_HEIGHT: f32 = 170.0;

impl Render for SpeedApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
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
                        download_color().into(),
                        self.phase == Phase::Download,
                    ))
                    .child(chart_panel(
                        "⬆  UPLOAD",
                        self.upload_mbps,
                        self.peak_upload,
                        &self.upload_samples,
                        upload_color().into(),
                        self.phase == Phase::Upload,
                    )),
            )
            .child(self.summary())
    }
}

impl SpeedApp {
    fn header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let running = self.running;
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

    let border = if active {
        color
    } else {
        rgb(PANEL_BORDER).into()
    };

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
