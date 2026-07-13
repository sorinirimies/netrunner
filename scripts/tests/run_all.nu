#!/usr/bin/env nu
# Test runner for netrunner_cli scripts.
# Validates script behaviour without needing a full Cargo build.
#
# Usage:  nu scripts/tests/run_all.nu
# Run from the repository root.

# ── Colour helpers ────────────────────────────────────────────────────────────
let green  = (ansi green)
let yellow = (ansi yellow)
let cyan   = (ansi cyan)
let red    = (ansi red)
let reset  = (ansi reset)

mut passed = 0
mut failed = 0

# ── Helper: report a single assertion ────────────────────────────────────────
def report [label: string, ok: bool, detail?: string] {
    if $ok {
        print $"  ($env.green)✔($env.reset) ($label)"
    } else {
        print $"  ($env.red)✘ FAIL($env.reset) ($label)"
        if not ($detail | is-empty) {
            print $"      ($env.yellow)→ ($detail)($env.reset)"
        }
    }
}

# Because `mut` variables cannot be captured inside `def`, we expose colours
# via $env for use inside the helper above.
$env.green  = $green
$env.yellow = $yellow
$env.cyan   = $cyan
$env.red    = $red
$env.reset  = $reset

# ─────────────────────────────────────────────────────────────────────────────
# Test suite 1 — version.nu
# Checks that the script prints a valid semver string that matches
# the version recorded in Cargo.toml.
# ─────────────────────────────────────────────────────────────────────────────
print ""
print $"($cyan)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"
print $"($cyan)  Suite 1 — version.nu($reset)"
print $"($cyan)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"

let semver_re = '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'

# 1a — scripts/version.nu exists
let version_script_exists = ("scripts/version.nu" | path exists)
if $version_script_exists {
    $passed += 1
} else {
    $failed += 1
}
report "scripts/version.nu exists" $version_script_exists "file not found"

# 1b — running the script outputs a non-empty string
let version_run = (do { run-external "nu" "scripts/version.nu" } | complete)
let script_output = $version_run.stdout | str trim

let script_ok = ($version_run.exit_code == 0) and (not ($script_output | is-empty))
if $script_ok {
    $passed += 1
} else {
    $failed += 1
}
report "version.nu exits 0 and prints output" $script_ok $"exit=($version_run.exit_code) output='($script_output)'"

# 1c — output matches valid semver
let output_is_semver = ($script_output =~ $semver_re)
if $output_is_semver {
    $passed += 1
} else {
    $failed += 1
}
report $"version.nu output is valid semver: '($script_output)'" $output_is_semver $"'($script_output)' does not match semver regex"

# 1d — output matches Cargo.toml package.version
let cargo_version = (open Cargo.toml | get workspace.package.version)
let versions_match = ($script_output == $cargo_version)
if $versions_match {
    $passed += 1
} else {
    $failed += 1
}
report $"version.nu matches Cargo.toml \(($cargo_version)\)" $versions_match $"script='($script_output)' cargo='($cargo_version)'"

# ─────────────────────────────────────────────────────────────────────────────
# Test suite 2 — version format regex
# Verifies that the regex used throughout the scripts accepts only valid semver.
# ─────────────────────────────────────────────────────────────────────────────
print ""
print $"($cyan)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"
print $"($cyan)  Suite 2 — version format regex($reset)"
print $"($cyan)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"

let valid_versions   = ["0.1.0" "1.2.3" "0.10.2" "1.0.0-beta.1"]
let invalid_versions = ["1.2" "1.2.3.4" "vv1.2.3" "abc"]

for v in $valid_versions {
    let ok = ($v =~ $semver_re)
    if $ok {
        $passed += 1
    } else {
        $failed += 1
    }
    report $"valid   '($v)' accepted" $ok $"regex rejected a valid version"
}

for v in $invalid_versions {
    let ok = ($v !~ $semver_re)
    if $ok {
        $passed += 1
    } else {
        $failed += 1
    }
    report $"invalid '($v)' rejected" $ok $"regex accepted an invalid version"
}

# ─────────────────────────────────────────────────────────────────────────────
# Test suite 3 — README badge replacement
# Ensures the badge regex replaces the version segment and nothing else.
# ─────────────────────────────────────────────────────────────────────────────
print ""
print $"($cyan)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"
print $"($cyan)  Suite 3 — README badge replacement regex($reset)"
print $"($cyan)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"

let badge_re      = 'version-[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?-blue'
let badge_input   = "![Version](https://img.shields.io/badge/version-0.6.1-blue)"
let badge_expect  = "![Version](https://img.shields.io/badge/version-1.0.0-blue)"
let badge_replace = "version-1.0.0-blue"

let badge_result = ($badge_input | str replace --all --regex $badge_re $badge_replace)

# 3a — output matches expected string
let badge_ok = ($badge_result == $badge_expect)
if $badge_ok {
    $passed += 1
} else {
    $failed += 1
}
report "badge regex produces correct replacement" $badge_ok $"got: '($badge_result)'  want: '($badge_expect)'"

# 3b — original URL structure is preserved (no extra replacements)
let url_preserved = ($badge_result | str contains "https://img.shields.io/badge/")
if $url_preserved {
    $passed += 1
} else {
    $failed += 1
}
report "badge replacement preserves URL structure" $url_preserved $"URL fragment missing from result: '($badge_result)'"

# 3c — old version string is gone
let old_gone = not ($badge_result | str contains "version-0.6.1-blue")
if $old_gone {
    $passed += 1
} else {
    $failed += 1
}
report "old version segment is removed from badge" $old_gone $"old segment still present in: '($badge_result)'"

# 3d — pre-release badge variant (e.g. -beta suffix)
let badge_pre_input  = "![Version](https://img.shields.io/badge/version-0.6.1-alpha-blue)"
let badge_pre_expect = "![Version](https://img.shields.io/badge/version-2.0.0-beta-blue)"
let badge_pre_replace = "version-2.0.0-beta-blue"
let badge_pre_result = ($badge_pre_input | str replace --all --regex $badge_re $badge_pre_replace)
let badge_pre_ok = ($badge_pre_result == $badge_pre_expect)
if $badge_pre_ok {
    $passed += 1
} else {
    $failed += 1
}
report "badge regex handles pre-release suffix" $badge_pre_ok $"got: '($badge_pre_result)'  want: '($badge_pre_expect)'"

# ─────────────────────────────────────────────────────────────────────────────
# Test suite 4 — Cargo.toml version regex
# Verifies that the package version line is updated and inline dependency
# version strings are left untouched.
# ─────────────────────────────────────────────────────────────────────────────
print ""
print $"($cyan)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"
print $"($cyan)  Suite 4 — Cargo.toml version regex \(package-only\)($reset)"
print $"($cyan)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"

let cargo_re = '(?m)^version = "[^"]*"'

# Representative Cargo.toml snippet that mirrors the real project layout:
# - A top-level `version = "..."` for the package
# - Inline dependency entries whose `version = "..."` fields must NOT change
let toml_input = '[package]
name = "netrunner_cli"
version = "0.6.1"
edition = "2021"

[dependencies]
clap = { version = "4.6", features = ["derive"] }
tokio = { version = "1.50", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
foo = { version = "1.0" }'

let toml_new_ver = "0.7.0"
let toml_result  = ($toml_input | str replace --regex $cargo_re $"version = \"($toml_new_ver)\"")

# 4a — the new package version appears in the result
let new_ver_present = ($toml_result | str contains $"version = \"($toml_new_ver)\"")
if $new_ver_present {
    $passed += 1
} else {
    $failed += 1
}
report "new package version present after replacement" $new_ver_present $"'version = \"($toml_new_ver)\"' not found in result"

# 4b — the old package version is gone
let old_pkg_gone = not ($toml_result | str contains "version = \"0.6.1\"")
if $old_pkg_gone {
    $passed += 1
} else {
    $failed += 1
}
report "old package version removed" $old_pkg_gone "old 'version = \"0.6.1\"' still present"

# 4c — inline dep `clap = { version = "4.6", ... }` is unchanged
let clap_intact = ($toml_result | str contains '{ version = "4.6"')
if $clap_intact {
    $passed += 1
} else {
    $failed += 1
}
report "inline clap version unchanged (\"4.6\")" $clap_intact "clap inline version was incorrectly modified"

# 4d — inline dep `tokio = { version = "1.50", ... }` is unchanged
let tokio_intact = ($toml_result | str contains '{ version = "1.50"')
if $tokio_intact {
    $passed += 1
} else {
    $failed += 1
}
report "inline tokio version unchanged (\"1.50\")" $tokio_intact "tokio inline version was incorrectly modified"

# 4e — inline dep `foo = { version = "1.0" }` is unchanged
let foo_intact = ($toml_result | str contains '{ version = "1.0" }')
if $foo_intact {
    $passed += 1
} else {
    $failed += 1
}
report "inline foo version unchanged (\"1.0\")" $foo_intact "foo inline version was incorrectly modified"

# 4f — exactly one replacement was made (regex has no --all flag)
#       Count occurrences of `version = "` in result vs input to confirm
let count_input  = ($toml_input  | split row 'version = "' | length) - 1
let count_result = ($toml_result | split row 'version = "' | length) - 1
let same_count   = ($count_input == $count_result)
if $same_count {
    $passed += 1
} else {
    $failed += 1
}
report $"replacement count unchanged \(($count_input) → ($count_result)\)" $same_count $"number of 'version = \"' occurrences changed: input=($count_input) result=($count_result)"

# ─────────────────────────────────────────────────────────────────────────────
# Final summary
# ─────────────────────────────────────────────────────────────────────────────
let total = $passed + $failed

print ""
print $"($cyan)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"

if $failed == 0 {
    print $"($green)  Results: ($passed)/($total) tests passed 🎉($reset)"
    print $"($green)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"
    print ""
} else {
    print $"($red)  Results: ($passed)/($total) passed, ($failed) FAILED ❌($reset)"
    print $"($red)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"
    print ""
    exit 1
}
