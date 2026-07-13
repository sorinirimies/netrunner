# VHS Terminal Recordings

This directory contains VHS tape files for creating automated terminal recordings of NetRunner CLI features.

## 📹 About VHS

[VHS](https://github.com/charmbracelet/vhs) (Video Home System) is a tool for generating terminal GIFs and videos from simple text-based scripts. It automates terminal interactions to create consistent, reproducible demos.

## 🎬 Available Recordings

### 1. speed-test.tape
**Basic Speed Test Demonstration**

Shows the complete speed test flow:
- IP-based geolocation detection
- Server discovery and selection
- Download and upload speed tests
- Real-time animated progress
- Final results display

Duration: ~60 seconds

### 2. debug-mode.tape
**Debug Mode Comparison**

Demonstrates the difference between normal and debug output:
- Clean output (default)
- Debug mode with trace logs
- Troubleshooting failed geolocation services

Duration: ~30 seconds

### 3. geolocation.tape
**Geolocation System Demo**

Highlights the robust geolocation features:
- 5 geolocation services with failover
- Data validation process
- ISP detection
- Smart fallback mechanism

Duration: ~45 seconds

### 4. history.tape
**History Management**

Shows test history tracking features:
- Running multiple tests
- Viewing recent test history
- Displaying statistics
- Automatic retention

Duration: ~90 seconds

### 5. json-output.tape
**JSON Output Integration**

Demonstrates machine-readable output:
- JSON format for automation
- Parsing with jq
- Saving to files
- CI/CD integration patterns

Duration: ~60 seconds

## 🚀 Quick Start

### Prerequisites

Install VHS:

```bash
# macOS
brew install vhs

# Linux (with Go)
go install github.com/charmbracelet/vhs@latest

# Or download binary from:
# https://github.com/charmbracelet/vhs/releases
```

### Generate Recordings

#### Using the Generate Script (Recommended)

```bash
# Navigate to VHS directory
cd examples/vhs

# Generate all recordings at once
./generate-all.sh

# Or clean and regenerate everything
./generate-all.sh --clean

# For help
./generate-all.sh --help
```

The script provides:
- ✓ Progress tracking with colored output
- ✓ Error handling and reporting
- ✓ File size and timing information
- ✓ Summary statistics

#### Manual Generation

```bash
# Navigate to VHS directory
cd examples/vhs

# Generate individual recordings
vhs speed-test.tape      # → speed-test.gif
vhs debug-mode.tape      # → debug-mode.gif
vhs geolocation.tape     # → geolocation.gif
vhs history.tape         # → history.gif
vhs json-output.tape     # → json-output.gif

# Or generate all at once with a loop
for tape in *.tape; do
    echo "Recording $tape..."
    vhs "$tape"
done
```

### Output

All GIFs will be generated in the `target/` directory (gitignored):
- `target/speed-test.gif` (800x600, ~3-5 MB)
- `target/debug-mode.gif` (600x400, ~2-3 MB)
- `target/geolocation.gif` (800x600, ~3-4 MB)
- `target/history.gif` (700x600, ~4-6 MB)
- `target/json-output.gif` (700x600, ~3-4 MB)

**Note:** The `target/` directory is gitignored, so generated GIFs won't be committed to version control.

## 🎨 Customization

### Modify Existing Tapes

Edit any `.tape` file to customize:

```vhs
# Appearance
Set Theme "Dracula"          # Try: Monokai, Nord, Solarized
Set FontSize 14              # Adjust for readability
Set Width 1200               # Terminal width in pixels
Set Height 800               # Terminal height in pixels

# Behavior  
Set PlaybackSpeed 0.5        # 0.5 = slower, 2.0 = faster
Set TypingSpeed 100ms        # Speed of typing animation

# Output
Output target/my-demo.gif
```

### Available Themes

- **Dracula** (default) - Dark purple theme
- **Monokai** - Classic dark theme
- **Nord** - Arctic blue theme
- **Solarized Dark/Light** - Elegant contrast
- **Catppuccin** - Pastel dark theme
- **Tokyo Night** - Modern dark theme

Full list: https://github.com/charmbracelet/vhs#themes

## 📝 Creating New Tapes

### Template

```vhs
# VHS documentation: https://github.com/charmbracelet/vhs

Output target/my-feature.gif

Set Shell "bash"
Set FontSize 14
Set Width 1200
Set Height 800
Set PlaybackSpeed 0.5
Set Theme "Dracula"
Set TypingSpeed 80ms

Hide
Type "cargo build --release"
Enter
Sleep 3s
Type "clear"
Enter
Show

Type "# My Feature Demo"
Sleep 1s
Enter
Enter

Type "./target/release/netrunner my-command"
Sleep 1s
Enter

# Wait for command to complete
Sleep 5s

Type "# Demo complete!"
Sleep 2s
```

### Best Practices

1. **Hide build steps** - Use `Hide`/`Show` to skip compilation
2. **Add context** - Include comments explaining what's happening
3. **Reasonable timing** - Balance between too fast and too slow
4. **Clean output** - Use `clear` to start with fresh terminal
5. **Add pauses** - Use `Sleep` after important information
6. **Test multiple times** - Ensure timing works consistently

## 🐛 Troubleshooting

### Common Issues

**Recording fails to start:**
```bash
# Check VHS is installed
vhs --version

# Verify tape file exists
ls -la *.tape
```

**Command not found in recording:**
```bash
# Build the binary first
cargo build --release

# Verify path in tape file matches
# Use absolute path if needed:
Type "/path/to/netrunner_cli/target/release/netrunner speed"
```

**Timing issues:**
```bash
# Increase Sleep durations for slower systems
Sleep 5s  # Instead of Sleep 2s

# Or adjust timeout in test config
# (if modifying source code)
```

**Output file too large:**
```bash
# Reduce dimensions
Set Width 1000   # Instead of 1200
Set Height 600   # Instead of 800

# Reduce duration
# Remove unnecessary Sleep commands
```

## 📖 VHS Documentation

- **Official Docs:** https://github.com/charmbracelet/vhs
- **Examples:** https://github.com/charmbracelet/vhs/tree/main/examples
- **Themes:** https://github.com/charmbracelet/vhs#themes

## 🤝 Contributing

Want to add a new recording?

1. Create a new `.tape` file following the existing format
2. Test it: `vhs your-demo.tape`
3. Add documentation to this README
4. Submit a pull request

### Recording Checklist

- [ ] Tape file named descriptively (`feature-name.tape`)
- [ ] Output path set correctly (`Output target/feature-name.gif`)
- [ ] Build steps hidden (`Hide` ... `Show`)
- [ ] Clear explanation comments
- [ ] Reasonable timing (not too fast/slow)
- [ ] Tested multiple times
- [ ] Documentation added to this README
- [ ] File size reasonable (<10 MB)

## 📄 License

All VHS tapes are licensed under MIT, same as the main project.