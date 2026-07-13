# VHS documentation: https://github.com/charmbracelet/vhs

Output "examples/vhs/target/speed-test.gif"

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 800
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

Type "# NetRunner CLI — speed test demo"
Sleep 500ms
Enter
Enter

Type "cargo run --release -- --no-animation --mode speed"
Sleep 400ms
Enter

# Geolocation + server discovery
Sleep 4s

# Server selection + latency probes
Sleep 3s

# Download test (~15 s)
Sleep 15s

# Upload test (~12 s)
Sleep 12s

# Final results
Sleep 3s

Escape
Sleep 500ms
