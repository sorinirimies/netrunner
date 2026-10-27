//! netrunner web dashboard (Topcoat). See the crate README.

use std::sync::atomic::{AtomicBool, Ordering};

use futures::{Stream, StreamExt};
use netrunner_core::{HistoryStorage, SpeedTest, SpeedTestResult, TestConfig, TestEvent};
use netrunner_web::{event_json, render_index};
use topcoat::{
    router::{
        content::{
            sse::{Event, KeepAlive, Sse},
            Html, Json,
        },
        route, Router, RouterBuilderDiscoverExt,
    },
    Result,
};

/// Only one speed test may run at a time: concurrent tests would skew each other.
static RUNNING: AtomicBool = AtomicBool::new(false);

/// Releases the [`RUNNING`] flag when the stream is dropped (finished or client gone).
struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        RUNNING.store(false, Ordering::SeqCst);
    }
}

fn recent(limit: usize) -> Vec<SpeedTestResult> {
    HistoryStorage::new()
        .and_then(|s| s.get_recent_results(limit))
        .unwrap_or_default()
}

#[route(GET "/")]
async fn index() -> Result<Html<String>> {
    Ok(Html(render_index(&recent(20))))
}

#[route(GET "/healthz")]
async fn healthz() -> Result<&'static str> {
    Ok("ok")
}

#[topcoat::router::query_params(error = redirect("?"))]
struct HistoryParams {
    limit: Option<usize>,
}

#[route(GET "/api/history")]
async fn history(cx: &topcoat::context::Cx) -> Result<Json<Vec<SpeedTestResult>>> {
    let params = topcoat::router::query_params::<HistoryParams>(cx)?;
    Ok(Json(recent(params.limit.unwrap_or(20).clamp(1, 500))))
}

#[route(GET "/api/run")]
async fn run() -> Result<Sse<impl Stream<Item = Result<Event>> + use<>>> {
    let busy = RUNNING.swap(true, Ordering::SeqCst);
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<(&'static str, serde_json::Value)>();

    if busy {
        let _ = tx.send((
            "busy",
            serde_json::json!({ "message": "A test is already running" }),
        ));
    } else {
        tokio::spawn(async move {
            let (core_tx, mut core_rx) = tokio::sync::mpsc::unbounded_channel::<TestEvent>();
            let fail = |msg: String| serde_json::json!({ "message": msg });
            let test = match SpeedTest::with_events(TestConfig::default(), Some(core_tx)) {
                Ok(t) => t,
                Err(e) => {
                    let _ = tx.send(("status", fail(format!("Failed to start: {e}"))));
                    return;
                }
            };
            let engine = test.run_full_test();
            tokio::pin!(engine);
            loop {
                tokio::select! {
                    biased;
                    ev = core_rx.recv() => match ev {
                        Some(ev) => {
                            if let TestEvent::Completed(result) = &ev {
                                if let Ok(store) = HistoryStorage::new() {
                                    let _ = store.save_result(result);
                                }
                            }
                            if let Some(out) = event_json(&ev) {
                                if tx.send(out).is_err() { return; } // client went away
                            }
                        }
                        None => break,
                    },
                    res = &mut engine => {
                        if let Err(e) = res {
                            let _ = tx.send(("status", fail(format!("Speed test failed: {e}"))));
                        }
                        break;
                    }
                }
            }
            while let Ok(ev) = core_rx.try_recv() {
                if let Some(out) = event_json(&ev) {
                    let _ = tx.send(out);
                }
            }
        });
    }

    let guard = (!busy).then_some(Guard);
    let stream = futures::stream::unfold((rx, guard), |(mut rx, guard)| async move {
        let (name, data) = rx.recv().await?;
        let event = Event::new().event(name).json_data(&data);
        Some((event, (rx, guard)))
    })
    .boxed();
    Ok(Sse::new(stream).keep_alive(KeepAlive::new()))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let listener = tokio::net::TcpListener::bind((host.as_str(), port)).await?;
    println!("netrunner web dashboard on http://{host}:{port}");
    topcoat::serve(listener, Router::builder().discover().build()).await
}
