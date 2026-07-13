# VHS documentation: https://github.com/charmbracelet/vhs

Output "examples/vhs/target/statistics-dashboard.gif"

Set Shell "bash"
Set FontSize 13
Set Width 1400
Set Height 900
Set PlaybackSpeed 1.0
Set Theme "Catppuccin Mocha"

# Build silently so the recording skips compilation
Hide
Type "cargo build --release --example statistics_dashboard 2>&1"
Enter
Sleep 30s
Type "clear"
Enter
Show

Type "# NetRunner CLI — statistics dashboard demo"
Sleep 400ms
Enter
Enter

Type "cargo run --release --example statistics_dashboard"
Sleep 400ms
Enter

# Binary seeds history and prints banner
Sleep 3s

# TUI opens — Download chart visible
Sleep 600ms

# Cycle to Upload
Tab
Sleep 600ms

# Cycle to Ping
Tab
Sleep 600ms

# Cycle to Quality
Tab
Sleep 600ms

# Back to Download
Tab
Sleep 500ms

# Scroll results down
Down
Sleep 200ms
Down
Sleep 200ms
Down
Sleep 200ms
Down
Sleep 200ms
Down
Sleep 200ms
Down
Sleep 200ms
Down
Sleep 200ms

# Pause on lower entries (Poor / VeryPoor / Failed)
Sleep 500ms

# Scroll back up
Up
Sleep 200ms
Up
Sleep 200ms
Up
Sleep 200ms
Up
Sleep 200ms
Up
Sleep 200ms
Up
Sleep 200ms
Up
Sleep 200ms

Sleep 400ms
Type "q"
Sleep 800ms
