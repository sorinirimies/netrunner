# netrunner-web

Browser dashboard for netrunner, built with [Topcoat](https://crates.io/crates/topcoat)
(server-side rendering) and server-sent events. The speed test runs on the machine hosting
the server; progress streams live to the page (gauge, live chart, phases) and results are
stored in the same history database the CLI and desktop app use.

```sh
cargo run -p netrunner-web          # http://127.0.0.1:3000
PORT=8080 HOST=0.0.0.0 cargo run -p netrunner-web --release
```

| Route | Purpose |
|-------|---------|
| `GET /` | dashboard (server-rendered history + live test UI) |
| `GET /api/run` | server-sent events: one speed test, streamed |
| `GET /api/history` | recent results as JSON (`?limit=N`, max 500) |
| `GET /healthz` | liveness probe |

Only one test runs at a time (a second `/api/run` gets a `busy` event).
Binds to loopback by default; set `HOST` explicitly to expose it.
