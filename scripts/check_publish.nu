#!/usr/bin/env nu
# Pre-publish readiness check for netrunner_cli.
# Runs formatting, linting, tests, doc build, file checks, and a dry-run
# publish to ensure the crate is ready to ship.
#
# Usage:  nu scripts/check_publish.nu
#
# Exit codes:
#   0 — all checks passed
#   1 — one or more checks failed

def main [] {
    let green  = (ansi green)
    let yellow = (ansi yellow)
    let cyan   = (ansi cyan)
    let red    = (ansi red)
    let reset  = (ansi reset)

    print $"($cyan)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"
    print $"($cyan)  netrunner_cli — pre-publish readiness check($reset)"
    print $"($cyan)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"
    print ""

    mut errors = 0

    # ─────────────────────────────────────────────────────────────────────────
    # Check 1 — cargo fmt
    # ─────────────────────────────────────────────────────────────────────────
    print $"($cyan)[1/8]($reset) Checking formatting \(cargo fmt -- --check\) …"
    let fmt = (
        do { run-external "cargo" "fmt" "--" "--check" }
        | complete
    )
    if $fmt.exit_code != 0 {
        print $"  ($red)✘ Formatting check failed.($reset) Run ($yellow)cargo fmt($reset) to fix."
        $errors = $errors + 1
    } else {
        print $"  ($green)✔ Formatting OK($reset)"
    }

    # ─────────────────────────────────────────────────────────────────────────
    # Check 2 — cargo clippy
    # ─────────────────────────────────────────────────────────────────────────
    print $"($cyan)[2/8]($reset) Running clippy \(--all-targets --all-features -D warnings -A deprecated\) …"
    let clippy = (
        do { run-external "cargo" "clippy" "--workspace" "--exclude" "netrunner" "--all-targets" "--all-features" "--" "-D" "warnings" "-A" "deprecated" }
        | complete
    )
    if $clippy.exit_code != 0 {
        print $"  ($red)✘ Clippy reported errors \(exit ($clippy.exit_code)\).($reset)"
        $errors = $errors + 1
    } else {
        print $"  ($green)✔ Clippy passed($reset)"
    }

    # ─────────────────────────────────────────────────────────────────────────
    # Check 3 — cargo test
    # ─────────────────────────────────────────────────────────────────────────
    print $"($cyan)[3/8]($reset) Running tests \(--all-features --all-targets\) …"
    let tests = (
        do { run-external "cargo" "test" "--workspace" "--exclude" "netrunner" "--all-features" "--all-targets" }
        | complete
    )
    if $tests.exit_code != 0 {
        print $"  ($red)✘ Tests failed \(exit ($tests.exit_code)\).($reset)"
        $errors = $errors + 1
    } else {
        print $"  ($green)✔ All tests passed($reset)"
    }

    # ─────────────────────────────────────────────────────────────────────────
    # Check 4 — cargo doc
    # ─────────────────────────────────────────────────────────────────────────
    print $"($cyan)[4/8]($reset) Building documentation \(--no-deps --all-features\) …"
    let docs = (
        do { run-external "cargo" "doc" "--no-deps" "--all-features" }
        | complete
    )
    if $docs.exit_code != 0 {
        print $"  ($red)✘ Documentation build failed \(exit ($docs.exit_code)\).($reset)"
        $errors = $errors + 1
    } else {
        print $"  ($green)✔ Documentation built successfully($reset)"
    }

    # ─────────────────────────────────────────────────────────────────────────
    # Check 5 — Required files present
    # ─────────────────────────────────────────────────────────────────────────
    print $"($cyan)[5/8]($reset) Checking required files …"
    let required_files = ["README.md" "LICENSE" "Cargo.toml" "CHANGELOG.md" "cliff.toml"]
    let missing_files = ($required_files | where { |f| not ($f | path exists) })
    if ($missing_files | is-empty) {
        print $"  ($green)✔ All required files present($reset)"
        for f in $required_files {
            print $"    ($green)·($reset) ($f)"
        }
    } else {
        for f in $missing_files {
            print $"  ($red)✘ Missing required file: ($f)($reset)"
        }
        $errors = $errors + ($missing_files | length)
    }

    # ─────────────────────────────────────────────────────────────────────────
    # Check 6 — Package version
    # ─────────────────────────────────────────────────────────────────────────
    print $"($cyan)[6/8]($reset) Checking package version …"
    let version = (open Cargo.toml | get workspace.package.version)
    if ($version | is-empty) {
        print $"  ($red)✘ Could not read workspace.package.version from Cargo.toml($reset)"
        $errors = $errors + 1
    } else {
        print $"  ($green)✔ Package version: ($yellow)($version)($reset)"
    }

    # ─────────────────────────────────────────────────────────────────────────
    # Check 7 — Cargo.lock present
    # ─────────────────────────────────────────────────────────────────────────
    print $"($cyan)[7/8]($reset) Checking Cargo.lock …"
    if ("Cargo.lock" | path exists) {
        print $"  ($green)✔ Cargo.lock present($reset)"
    } else {
        print $"  ($red)✘ Cargo.lock not found.($reset) Run ($yellow)cargo generate-lockfile($reset)"
        $errors = $errors + 1
    }

    # ─────────────────────────────────────────────────────────────────────────
    # Check 8 — cargo publish --dry-run
    # ─────────────────────────────────────────────────────────────────────────
    print $"($cyan)[8/8]($reset) Running cargo publish --dry-run for netrunner-core …"
    let dry_run = (
        do { run-external "cargo" "publish" "-p" "netrunner-core" "--dry-run" "--allow-dirty" }
        | complete
    )
    if $dry_run.exit_code != 0 {
        print $"  ($red)✘ Publish dry-run failed \(exit ($dry_run.exit_code)\).($reset)"
        $errors = $errors + 1
    } else {
        print $"  ($green)✔ Publish dry-run passed($reset)"
    }

    # ─────────────────────────────────────────────────────────────────────────
    # Summary
    # ─────────────────────────────────────────────────────────────────────────
    print ""
    if $errors == 0 {
        print $"($green)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"
        print $"($green)  All checks passed! 🎉  Ready to publish ($version)($reset)"
        print $"($green)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"
        print ""
        print $"($cyan)Next steps:($reset)"
        print $"  Bump version and tag:    ($yellow)nu scripts/bump_version.nu ($version)($reset)"
        print $"  Publish core crate:      ($yellow)cargo publish -p netrunner-core($reset)"
        print $"  Then the CLI:            ($yellow)cargo publish -p netrunner_cli($reset)"
        print ""
    } else {
        print $"($red)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"
        print $"($red)  ($errors) check\(s\) failed. Fix the issues above before publishing.($reset)"
        print $"($red)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"
        print ""
        exit 1
    }
}
