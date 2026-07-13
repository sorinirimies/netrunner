# Netrunner 🚀

A fast, cyberpunk-styled internet **speed test & network diagnostics** toolkit in Rust — as a terminal app, a desktop app, and an embeddable library.

[![crates.io](https://img.shields.io/crates/v/netrunner_cli?label=netrunner_cli)](https://crates.io/crates/netrunner_cli)
[![core docs](https://img.shields.io/docsrs/netrunner-core?label=netrunner-core%20docs)](https://docs.rs/netrunner-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

```
┌ netrunner-core ┐        the engine — no UI, embeddable
│  speed test    │──┬──▶  netrunner_cli   (Ratatui TUI)
│  diagnostics   │  └──▶  netrunner       (Zed GPUI desktop app)
│  history       │
└────────────────┘
```

All network logic lives in one framework-free core crate; both front-ends
subscribe to the same progress-event stream, so behaviour is identical across
the terminal and the desktop.

## Preview

| Speed test | Statistics dashboard | History |
|------------|----------------------|---------|
| ![speed test](crates/netrunner-cli/examples/vhs/target/speed-test.gif) | ![statistics dashboard](crates/netrunner-cli/examples/vhs/target/statistics-dashboard.gif) | ![history](crates/netrunner-cli/examples/vhs/target/history.gif) |

## Install

```sh
cargo install netrunner_cli     # terminal app  → `netrunner_cli`
cargo install netrunner         # desktop app   → `netrunner`
cargo add netrunner-core        # embed the engine in your own project
```

---

## Terminal app — `netrunner_cli`

```sh
netrunner_cli                 # interactive menu
netrunner_cli -m speed        # run a speed test
netrunner_cli -m diag         # network diagnostics
netrunner_cli -m full         # speed test + diagnostics
netrunner_cli --history       # statistics dashboard (pie charts)
netrunner_cli --json          # machine-readable output (no TUI)
```

| Flag | Description | Default |
|------|-------------|---------|
| `-m, --mode` | `speed`, `diag`, `history`, `full`, `servers` | `speed` |
| `-s, --server` | Custom test server URL | auto |
| `-t, --timeout` | Per-test timeout (seconds) | `30` |
| `-d, --detail` | `basic`, `standard`, `detailed`, `debug` | `standard` |
| `-j, --json` | JSON output, skip the TUI | off |
| `-n, --no-animation` | Disable animations | off |

Live download/upload bandwidth graphs render while the test runs, followed by a
results panel and — with `--history` — a full-screen dashboard of pie charts.

## Desktop app — `netrunner`

A [Zed **GPUI**](https://www.gpui.rs/) desktop app that runs the same engine and
draws **live download/upload bar charts** as the test progresses, plus a summary
of ping, throughput, connection quality, location, ISP and server.

```sh
cargo run -p netrunner        # from a clone
netrunner                     # after `cargo install netrunner`
```

> The GUI needs a GPUI-capable platform (macOS, or Linux with the usual
> Vulkan/Wayland/X11 libraries). The core engine and TUI are fully headless.

## Library — `netrunner-core`

Framework-free engine with **no UI dependencies**. Drop it into any async Rust
project to measure connections, run diagnostics, or persist history.

```rust
use netrunner_core::{SpeedTest, TestConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test = SpeedTest::new(TestConfig::default())?;
    let r = test.run_full_test().await?;

    println!("↓ {:.1} Mbps  ↑ {:.1} Mbps  ping {:.0} ms  [{}]",
        r.download_mbps, r.upload_mbps, r.ping_ms, r.quality);
    Ok(())
}
```

### Stream live progress

The engine is UI-agnostic: pass a Tokio channel and it emits
[`TestEvent`]s (location, server selection, download/upload samples, latency,
completion) that you render however you like.

```rust
use netrunner_core::{SpeedTest, TestConfig, TestEvent};
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::unbounded_channel::<TestEvent>();
let test = SpeedTest::with_events(TestConfig::default(), Some(tx))?;

tokio::spawn(async move { let _ = test.run_full_test().await; });

while let Some(event) = rx.recv().await {
    if let TestEvent::DownloadSample { mbps, .. } = event {
        println!("download: {mbps:.1} Mbps");
    }
}
```

### Diagnostics & history

```rust
use netrunner_core::{HistoryStorage, NetworkDiagnosticsTool, TestConfig};

// Gateway, DNS, route, IPv6, interface
let diag = NetworkDiagnosticsTool::new(TestConfig::default())
    .run_diagnostics().await?;

// Embedded redb store with rolling statistics
let store = HistoryStorage::new()?;
store.save_result(&result)?;
let stats = store.get_statistics()?;
```

Public surface: `SpeedTest`, `TestConfig`, `TestEvent`, `NetworkDiagnosticsTool`,
`HistoryStorage`, `SpeedTestResult`, `ConnectionQuality`, and more — see
[docs.rs/netrunner-core](https://docs.rs/netrunner-core).

---

## How it works

- **Throughput** — up to 50 parallel connections with large chunks and a warmup
  window, sampled progressively and averaged (supports gigabit+).
- **Server selection** — IP geolocation (5 providers with failover) → dynamic
  server discovery → Haversine distance + latency scoring → best 3 servers.
- **Diagnostics** — gateway, DNS servers & response time, route hops, IPv6,
  interface.
- **History** — embedded [redb](https://crates.io/crates/redb) database with
  30-day retention and per-run statistics.

## Development

This is a Cargo workspace driven by a [`justfile`](justfile):

```sh
just build            # build everything
just run-tui          # run the terminal app
just run-gui          # run the desktop app
just test             # full workspace test suite
just test-headless    # core + CLI only (no GPUI, mirrors CI)
just release 1.2.3    # gate → bump → tag → push → publish (via CI)
```

| Crate | Package | Kind |
|-------|---------|------|
| `crates/netrunner-core` | `netrunner-core` | library (embeddable engine) |
| `crates/netrunner-cli` | `netrunner_cli` | binary (Ratatui TUI) |
| `crates/netrunner-gui` | `netrunner` | binary (Zed GPUI desktop) |

## License

MIT © [Sorin Albu-Irimies](https://github.com/sorinirimies) — see [LICENSE](LICENSE).
