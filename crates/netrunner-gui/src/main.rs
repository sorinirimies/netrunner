//! Desktop entry point for the netrunner GPUI app (macOS and Linux).

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn main() {
    use gpui::{
        App, AppContext, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions, px,
        size,
    };
    use netrunner_gui::SpeedApp;

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
                app.update(cx, |state, cx| {
                    // Load persisted settings + history, then auto-start a test
                    // if the user enabled it (default on) so the charts come alive.
                    state.boot();
                    if state.settings.auto_run {
                        state.start(cx);
                    }
                });
                app
            },
        )
        .expect("open the main window");
        cx.activate(true);
    });
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn main() {
    eprintln!(
        "netrunner: the desktop app runs on macOS and Linux. Use netrunner_cli (terminal) or netrunner-web (browser) here."
    );
    std::process::exit(1);
}
