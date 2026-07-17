#!/usr/bin/env nu
# Tests for scripts/bump_version.nu — the pure string/regex transforms:
# semver validation, README badge bump, and the Cargo.toml version rewrite
# (workspace version + internal netrunner-core pin, leaving 3rd-party deps).

use std/assert
use runner.nu *

# ── semver validation (mirrors the regex in bump_version.nu) ─────────────────
def valid_semver [v: string]: nothing -> bool {
    $v =~ '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'
}

def "test bump: accepts valid semver" [] {
    for v in ["1.2.3" "0.0.0" "10.20.30" "1.0.0-beta.1" "2.0.0-rc.2"] {
        assert (valid_semver $v)
    }
}

def "test bump: rejects invalid semver" [] {
    for v in ["1.2" "v1.2.3" "1.2.3.4" "notaversion" "1.2.x"] {
        assert (not (valid_semver $v))
    }
}

# ── README badge bump ────────────────────────────────────────────────────────
def bump_badge [readme: string, new: string]: nothing -> string {
    $readme | str replace --all --regex 'version-[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?-blue' $"version-($new)-blue"
}

def "test bump: badge replacement" [] {
    assert equal (bump_badge "img version-1.0.0-blue end" "2.0.0") "img version-2.0.0-blue end"
}

def "test bump: badge handles pre-release" [] {
    assert equal (bump_badge "version-1.0.0-rc1-blue" "2.0.0") "version-2.0.0-blue"
}

# ── Cargo.toml rewrite (workspace version + internal core pin) ───────────────
def bump_cargo [toml: string, new: string]: nothing -> string {
    $toml
    | str replace --regex '(?m)^version = "[^"]*"' $"version = \"($new)\""
    | str replace --all --regex 'netrunner-core = \{ path = "crates/netrunner-core", version = "[^"]*" \}' $"netrunner-core = { path = \"crates/netrunner-core\", version = \"($new)\" }"
}

def sample_cargo []: nothing -> string {
    '[workspace.package]
version = "1.0.0"

[workspace.dependencies]
clap = { version = "4.6", features = ["derive"] }
tokio = { version = "1.50" }
netrunner-core = { path = "crates/netrunner-core", version = "1.0.0" }
'
}

def "test bump: updates workspace version" [] {
    let out = (bump_cargo (sample_cargo) "2.1.0")
    assert ($out | str contains 'version = "2.1.0"')
    # No stale 1.0.0 left (workspace + internal pin both moved).
    assert (not ($out | str contains 'version = "1.0.0"'))
}

def "test bump: updates internal core dep pin" [] {
    let out = (bump_cargo (sample_cargo) "2.1.0")
    assert ($out | str contains 'netrunner-core = { path = "crates/netrunner-core", version = "2.1.0" }')
}

def "test bump: leaves third-party deps untouched" [] {
    let out = (bump_cargo (sample_cargo) "2.1.0")
    assert ($out | str contains '{ version = "4.6"')
    assert ($out | str contains '{ version = "1.50" }')
}

def main [] { run-tests }
