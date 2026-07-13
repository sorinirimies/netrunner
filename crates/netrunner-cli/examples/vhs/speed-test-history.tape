# VHS documentation: https://github.com/charmbracelet/vhs

Output "examples/vhs/target/history.gif"

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 700
Set PlaybackSpeed 1.0
Set Theme "Catppuccin Mocha"

# Build silently so the recording skips compilation
Hide
Type "cargo build --release 2>&1"
Enter
Sleep 30s
Type "clear"
Enter
Show

Type "# NetRunner CLI — test history & statistics dashboard"
Sleep 400ms
Enter
Enter

# Run a quick speed test first to populate history
Type "cargo run --release -- --no-animation --mode speed"
Sleep 400ms
Enter

# Geolocation + server discovery
Sleep 4s

# Server selection
Sleep 3s

# Download test
Sleep 15s

# Upload test
Sleep 12s

# Results
Sleep 3s

Type "clear"
Enter
Sleep 300ms

# Now open the statistics dashboard
Type "cargo run --release -- --history"
Sleep 400ms
Enter

# TUI boot
Sleep 2s

# View Download chart
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

Sleep 500ms
Type "q"
Sleep 800ms
