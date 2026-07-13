# netrunner workspace — task runner
# Install just:      cargo install just
# Install vhs:       brew install vhs  OR  go install github.com/charmbracelet/vhs@latest
# Install git-cliff: cargo install git-cliff
# Usage: just <task>
# ── Default ───────────────────────────────────────────────────────────────────

default:
    @just --list

# ── Tool checks ───────────────────────────────────────────────────────────────

_check-git-cliff:
    @command -v git-cliff >/dev/null 2>&1 || { \
        echo "❌ git-cliff not found. Install with: cargo install git-cliff"; exit 1; \
    }

# Check nu (nushell) is available
_check-nu:
    @command -v nu >/dev/null 2>&1 || { \
        echo "❌ nu (nushell) not found. Install: https://www.nushell.sh"; exit 1; \
    }

_check-vhs:
    @command -v vhs >/dev/null 2>&1 || { \
        echo "❌ vhs not found."; \
        echo "   macOS:      brew install vhs"; \
        echo "   Any:        go install github.com/charmbracelet/vhs@latest"; \
        exit 1; \
    }

# Install all recommended development tools
install-tools:
    @echo "Installing development tools…"
    @command -v git-cliff >/dev/null 2>&1 || cargo install git-cliff --locked
    @command -v nu >/dev/null 2>&1 || cargo install nu --locked
    @echo "✅ All tools installed!"

# ── Build ─────────────────────────────────────────────────────────────────────

# Build the entire workspace (dev)
build:
    cargo build --workspace

# Build only the core library (dev)
build-core:
    cargo build -p netrunner-core

# Build only the GPUI desktop GUI (dev)
build-gui:
    cargo build -p netrunner

# Build only the TUI crate (dev)
build-tui:
    cargo build -p netrunner_cli

# Build release binaries for GUI and TUI
build-release:
    cargo build --release -p netrunner
    cargo build --release -p netrunner_cli

# ── Run ───────────────────────────────────────────────────────────────────────

# Launch the Zed GPUI desktop app (live download/upload charts)
run-gui:
    cargo run -p netrunner

# Launch the Ratatui terminal UI
run-tui:
    cargo run -p netrunner_cli

# Alias: default run launches the GUI
run: run-gui

# ── Test ──────────────────────────────────────────────────────────────────────

# Run the full workspace test suite (needs a GPUI-capable machine, e.g. macOS)
test:
    cargo test --workspace --locked --all-features --all-targets

# Test only the core library
test-core:
    cargo test -p netrunner-core --all-features

# Test only the GPUI desktop app
test-gui:
    cargo test -p netrunner --all-features

# Test only the TUI crate
test-tui:
    cargo test -p netrunner_cli --all-features

# Run only the headless (CI-friendly) crates: core + TUI, no GPUI
test-headless:
    cargo test --workspace --exclude netrunner --locked --all-features --all-targets

# Run Nu script tests
test-nu: _check-nu
    nu scripts/tests/run_all.nu

# Run both Rust and Nu tests
test-all-nu: test test-nu
    @echo "✅ All Rust and Nu tests passed!"

# ── Code quality ──────────────────────────────────────────────────────────────

# Check without building
check:
    cargo check --workspace

# Format all code
fmt:
    cargo fmt --all

# Check formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Run clippy across the workspace
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings -A deprecated

# Clippy for the headless crates only (mirrors CI: no GPUI on Linux)
clippy-headless:
    cargo clippy --workspace --exclude netrunner --all-targets --all-features -- -D warnings -A deprecated

# Run all quality checks (format, clippy, test, nu) — must pass before a release.

# Auto-formats first, then verifies no changes remain (catches unstaged format diffs).
check-all: fmt clippy test test-nu
    @echo "🔍 Verifying formatting is clean…"
    cargo fmt --all -- --check
    @echo "✅ All checks passed!"

# Full pre-release quality gate — everything in check-all plus a release build.
check-release: check-all build-release
    @echo "✅ Release quality gate passed (fmt + clippy + test + nu + release build)!"

# ── VHS Demo GIFs ─────────────────────────────────────────────────────────────

GUI_VHS := "crates/netrunner-gui/examples/vhs"
TUI_VHS := "crates/netrunner-cli/examples/vhs"
GUI_VHS_GENERATED := "crates/netrunner-gui/examples/vhs/generated"
TUI_VHS_GENERATED := "crates/netrunner-cli/examples/vhs/target"

# Generate all VHS demo GIFs (GUI + TUI)
vhs-all: vhs-gui vhs-tui

# Generate only the GUI demo GIFs
vhs-gui: _check-vhs
    #!/usr/bin/env sh
    set -e
    mkdir -p {{ GUI_VHS_GENERATED }}
    echo "╔════════════════════════════════════════════╗"
    echo "║   GUI Tapes (GPUI desktop)                 ║"
    echo "╚════════════════════════════════════════════╝"
    for tape in {{ GUI_VHS }}/*.tape; do
        [ -f "$tape" ] || continue
        echo "▶  $tape"
        vhs "$tape" || echo "❌ Failed: $tape"
    done
    echo "✅ GUI demos done → {{ GUI_VHS_GENERATED }}/"

# Generate only the TUI demo GIFs
vhs-tui: _check-vhs
    #!/usr/bin/env sh
    set -e
    mkdir -p {{ TUI_VHS_GENERATED }}
    echo "╔════════════════════════════════════════════╗"
    echo "║   TUI Tapes (Ratatui terminal)            ║"
    echo "╚════════════════════════════════════════════╝"
    for tape in {{ TUI_VHS }}/*.tape; do
        [ -f "$tape" ] || continue
        echo "▶  $tape"
        vhs "$tape" || echo "❌ Failed: $tape"
    done
    echo "✅ TUI demos done → {{ TUI_VHS_GENERATED }}/"

# Render a single tape by name (e.g. just vhs-tape speed-test)
vhs-tape name: _check-vhs
    #!/usr/bin/env sh
    if [ -f "{{ GUI_VHS }}/{{ name }}.tape" ]; then
        echo "▶  {{ GUI_VHS }}/{{ name }}.tape"
        vhs "{{ GUI_VHS }}/{{ name }}.tape" && echo "✅ Done."
    elif [ -f "{{ TUI_VHS }}/{{ name }}.tape" ]; then
        echo "▶  {{ TUI_VHS }}/{{ name }}.tape"
        vhs "{{ TUI_VHS }}/{{ name }}.tape" && echo "✅ Done."
    else
        echo "❌ Tape not found: {{ name }}.tape"
        echo ""
        just vhs-list
        exit 1
    fi

# List all available VHS tapes
vhs-list:
    #!/usr/bin/env sh
    echo "GUI tapes  →  {{ GUI_VHS }}/"
    ls {{ GUI_VHS }}/*.tape 2>/dev/null | sed 's|.*/||; s|\.tape||' | sed 's/^/  /' || echo "  (none)"
    echo ""
    echo "TUI tapes  →  {{ TUI_VHS }}/"
    ls {{ TUI_VHS }}/*.tape 2>/dev/null | sed 's|.*/||; s|\.tape||' | sed 's/^/  /' || echo "  (none)"

# ── Documentation ─────────────────────────────────────────────────────────────

# Generate and open docs for the core library
doc-core:
    cargo doc --no-deps -p netrunner-core --open

# Generate and open docs for the GUI crate
doc-gui:
    cargo doc --no-deps -p netrunner --open

# Generate and open docs for the TUI crate
doc-tui:
    cargo doc --no-deps -p netrunner_cli --open

# Generate docs for the full workspace (no browser)
doc:
    cargo doc --no-deps --workspace

# ── Changelog ─────────────────────────────────────────────────────────────────

# Regenerate the full CHANGELOG.md from all tags
changelog: _check-git-cliff
    @echo "Generating full changelog…"
    git-cliff --output CHANGELOG.md
    @echo "✅ CHANGELOG.md updated."

# Prepend only unreleased commits to CHANGELOG.md
changelog-unreleased: _check-git-cliff
    git-cliff --unreleased --prepend CHANGELOG.md
    @echo "✅ Unreleased changes prepended."

# Preview changelog for the next release without writing the file
changelog-preview: _check-git-cliff
    @git-cliff --unreleased

# Show the latest tagged release entry (no file write)
changelog-latest: _check-git-cliff
    @git-cliff --latest

# ── Version bump ─────────────────────────────────────────────────────────────

# Fail fast if the requested version is the same as the current one.
_check-version-changed version: _check-nu
    #!/usr/bin/env sh
    current=$(nu scripts/version.nu)
    if [ "$current" = "{{ version }}" ]; then
        echo "❌ Version {{ version }} is already the current version. Nothing to bump."
        exit 1
    fi
    echo "✅ Version will change: $current → {{ version }}"

# Bump the workspace version, regenerate Cargo.lock + CHANGELOG.md, commit, tag and push.

# Validation runs first (cheap), quality gate runs second (expensive).
bump version: (_check-version-changed version) check-release _check-git-cliff
    nu scripts/bump_version.nu {{ version }}

# ── Publish (crates.io) ───────────────────────────────────────────────────────

# Run the full pre-publish readiness check (fmt, clippy, tests, docs, dry-run)
check-publish: _check-nu
    nu scripts/check_publish.nu

# Dry-run publish for the core library (CLI/GUI can only be verified once core
# is on crates.io, because they depend on it by version).
publish-dry: check-all
    @echo "Dry-run: netrunner-core"
    cargo publish --dry-run -p netrunner-core

# Publish all three in dependency order: core → CLI → GUI.
publish: check-all publish-core publish-cli publish-gui
    @echo "✅ netrunner-core, netrunner_cli, and netrunner published to crates.io!"

# Publish netrunner-core (required by the CLI and GUI)
publish-core:
    @echo "📦 Publishing netrunner-core…"
    cargo publish -p netrunner-core
    @echo "⏳ Waiting 30 s for the index to propagate…"
    sleep 30

# Publish netrunner_cli (the TUI — the established crates.io package)
publish-cli:
    @echo "📦 Publishing netrunner_cli (TUI)…"
    cargo publish -p netrunner_cli

# Publish netrunner (the GPUI desktop app) — run from a GPUI-capable machine.
publish-gui:
    @echo "📦 Publishing netrunner (GUI)…"
    cargo publish -p netrunner

# Show what would be released without making any changes
release-preview: _check-git-cliff
    @echo "Current version: $(just version)"
    @echo ""
    @echo "Unreleased commits:"
    @git-cliff --unreleased
    @echo ""
    @echo "Workspace version:"
    @grep -A5 '^\[workspace\.package\]' Cargo.toml | grep '^version'
    @echo ""
    @echo "Published crates:  netrunner-core  •  netrunner_cli (TUI)  •  netrunner (GUI)"

# ── Housekeeping ──────────────────────────────────────────────────────────────

# Remove build artifacts
clean:
    cargo clean

# Update all dependencies (Cargo.lock only)
update:
    cargo update

# Update dependencies, run the full quality gate, then commit and push if all green.
update-deps:
    @echo "⬆️  Updating dependencies…"
    cargo update
    @echo "🔍 Running quality gate…"
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings -A deprecated
    cargo test --workspace --locked --all-features --all-targets
    @echo "✅ All checks passed — committing dependency updates…"
    git add Cargo.lock
    git diff --cached --quiet || git commit -m "chore: update dependencies"
    git push origin main
    @echo "✅ Dependency updates pushed to GitHub."

# Show outdated dependencies (requires cargo-outdated)
outdated:
    cargo outdated

# Show the current workspace version
version: _check-nu
    @nu scripts/version.nu

# Show all configured remotes
remotes:
    @git remote -v

# ── Git remotes & pushing ────────────────────────────────────────────────────

# Push the current branch to GitHub (origin)
push:
    git push origin main

# Push the current branch to Gitea
push-gitea:
    git push gitea main

# Push the current branch to all remotes (continues on failure)
push-all:
    #!/usr/bin/env sh
    failed=""
    git push origin main   || failed="$failed origin"
    git push gitea main    || failed="$failed gitea"
    if [ -n "$failed" ]; then
        echo "⚠️  Failed to push to:$failed"
    else
        echo "✅ Pushed to GitHub and Gitea!"
    fi

# Pull the current branch from GitHub (origin)
pull:
    git pull origin main

# Pull the current branch from Gitea
pull-gitea:
    git pull gitea main

# Push all tags to GitHub
push-tags:
    git push origin --tags

# Push all tags to all remotes (continues on failure)
push-tags-all:
    #!/usr/bin/env sh
    failed=""
    git push origin --tags   || failed="$failed origin"
    git push gitea --tags    || failed="$failed gitea"
    if [ -n "$failed" ]; then
        echo "⚠️  Failed to push tags to:$failed"
    else
        echo "✅ Tags pushed to all remotes!"
    fi

# ── Release workflows ─────────────────────────────────────────────────────────

# Bump, commit, tag, then push to GitHub — triggers the Release workflow.
release version: (bump version)
    @echo "Pushing release v{{ version }} to GitHub…"
    git push --follow-tags origin main
    @echo "✅ Release v{{ version }} pushed — the Release workflow will trigger automatically."
    @echo "   https://github.com/$(git remote get-url origin | sed 's/.*github.com[:/]//' | sed 's/\.git//')/actions"

# Bump, commit, tag, then push to all remotes (continues on failure).
release-all version: (bump version)
    #!/usr/bin/env sh
    echo "Pushing release v{{ version }} to all remotes…"
    failed=""
    git push --follow-tags origin main   || failed="$failed origin"
    git push --follow-tags gitea main    || failed="$failed gitea"
    if [ -n "$failed" ]; then
        echo "⚠️  Release v{{ version }} failed to push to:$failed"
    else
        echo "✅ Release v{{ version }} pushed to GitHub and Gitea!"
    fi

# Re-trigger the Release workflow for an EXISTING tag via manual dispatch.
# Uses workflow_dispatch (reliable) — handy after fixing a secret such as
# CRATES_IO_TOKEN. Requires the GitHub CLI (gh).
release-retrigger version:
    @command -v gh >/dev/null 2>&1 || { \
        echo "❌ GitHub CLI (gh) not found. Install from https://cli.github.com"; exit 1; \
    }
    @echo "Dispatching Release workflow for tag v{{ version }}…"
    gh workflow run release.yml --field tag=v{{ version }}
    @echo "✅ Dispatched — check progress at:"
    @echo "   https://github.com/$(gh repo view --json nameWithOwner -q .nameWithOwner)/actions"

# Force-sync Gitea with GitHub
sync-gitea:
    git push gitea main --force
    git push gitea --tags --force
    @echo "✅ Gitea force-synced with GitHub."

# Add a Gitea remote and optionally push — interactive (nu script)
setup-gitea url: _check-nu
    nu scripts/setup_gitea.nu {{ url }}

# Migrate this project to dual GitHub + Gitea hosting (interactive)
migrate-gitea: _check-nu
    nu scripts/migrate_to_gitea.nu
