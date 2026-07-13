# netrunner_cli

A feature-rich, cyberpunk-styled terminal app to test and analyze your internet
connection. The network logic lives in
[`netrunner-core`](../netrunner-core); this crate owns the Ratatui /
`indicatif` / `colored` presentation layer, including the animated logo intro,
the live download/upload bandwidth graphs and the pie-chart statistics
dashboard.

## Usage

```sh
netrunner_cli                 # interactive menu
netrunner_cli --mode speed    # run a speed test
netrunner_cli --mode diag     # network diagnostics
netrunner_cli --history       # statistics dashboard
netrunner_cli --json          # machine-readable output
```
