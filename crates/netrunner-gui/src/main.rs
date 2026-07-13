//! Desktop entry point for the netrunner GPUI app.

use gpui::{
    px, size, App, AppContext, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions,
};
use netrunner_gui::SpeedApp;

fn main() {
    // Install a rustls crypto provider for the whole process.
    let _ = rustls::crypto::ring::default_provider().install_default();

    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(980.0), px(680.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("netrunner — Speed Test".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| {
                let app = cx.new(|_| SpeedApp::new());
                // Auto-start a test so the charts come alive immediately.
                app.update(cx, |state, cx| state.start(cx));
                app
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
