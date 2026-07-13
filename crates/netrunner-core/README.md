# netrunner-core

Framework-free internet speed-test & network-diagnostics engine shared by the
[`netrunner_cli`](../netrunner-cli) terminal app and the
[`netrunner`](../netrunner-gui) GPUI desktop app.

It contains **no UI code**. Long-running operations report progress through
`TestEvent`s streamed over a Tokio channel, so any front-end can render them.

## Modules

| Module | Responsibility |
|--------|----------------|
| `types` | Domain models — results, servers, config, quality ratings, diagnostics |
| `speed_test` | Geolocation, server discovery/selection, throughput & latency measurement |
| `diagnostics` | Gateway/DNS/route/IPv6 network diagnostics |
| `history` | Embedded `redb` history storage & statistics |
| `events` | UI-agnostic progress events (`TestEvent`) |

## Example

```rust,no_run
use netrunner_core::{SpeedTest, TestConfig};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let test = SpeedTest::new(TestConfig::default())?;
let result = test.run_full_test().await?;
println!("{:.1} down / {:.1} up", result.download_mbps, result.upload_mbps);
# Ok(())
# }
```
