//! Bridges the async, Tokio-based [`netrunner_core`] engine to GPUI.
//!
//! GPUI runs its own (non-Tokio) executor, while `netrunner-core` uses `reqwest`
//! which requires a Tokio runtime. We therefore run the speed test on a
//! dedicated background thread with its own multi-threaded Tokio runtime and
//! stream [`TestEvent`]s back over a `futures` channel that GPUI can poll from
//! its foreground executor.

use futures::channel::mpsc::{unbounded, UnboundedReceiver};
use netrunner_core::{SpeedTest, TestConfig, TestEvent};

/// Start a speed test on a background Tokio runtime.
///
/// Returns a stream of [`TestEvent`]s. The test begins immediately; drop the
/// receiver to stop caring about its progress (the background thread finishes
/// on its own).
pub fn spawn_speed_test(config: TestConfig) -> UnboundedReceiver<TestEvent> {
    let (ui_tx, ui_rx) = unbounded::<TestEvent>();

    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                let _ = ui_tx
                    .unbounded_send(TestEvent::Status(format!("Failed to start runtime: {e}")));
                return;
            }
        };

        runtime.block_on(async move {
            // Ensure a rustls crypto provider is installed for this runtime.
            let _ = rustls::crypto::ring::default_provider().install_default();

            let (core_tx, mut core_rx) = tokio::sync::mpsc::unbounded_channel::<TestEvent>();
            let test = match SpeedTest::with_events(config, Some(core_tx)) {
                Ok(test) => test,
                Err(e) => {
                    let _ = ui_tx.unbounded_send(TestEvent::Status(format!(
                        "Failed to create speed test: {e}"
                    )));
                    return;
                }
            };

            let engine = test.run_full_test();
            tokio::pin!(engine);

            loop {
                tokio::select! {
                    biased;
                    maybe = core_rx.recv() => match maybe {
                        Some(event) => {
                            if ui_tx.unbounded_send(event).is_err() {
                                break; // GUI dropped the receiver.
                            }
                        }
                        None => break,
                    },
                    res = &mut engine => {
                        if let Err(e) = res {
                            let _ = ui_tx.unbounded_send(TestEvent::Status(format!(
                                "Speed test failed: {e}"
                            )));
                        }
                        break;
                    }
                }
            }

            // Forward any events buffered after the engine finished.
            while let Ok(event) = core_rx.try_recv() {
                let _ = ui_tx.unbounded_send(event);
            }
        });
    });

    ui_rx
}
