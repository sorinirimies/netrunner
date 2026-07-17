#!/usr/bin/env nu
# Tests for scripts/version.nu — reading the workspace version from Cargo.toml.

use std/assert
use runner.nu *

# Replicate version.nu's logic against an in-memory TOML string.
def read_ws_version [toml: string]: nothing -> string {
    $toml | from toml | get workspace.package.version
}

def "test version: reads workspace.package.version" [] {
    let toml = '[workspace.package]
version = "1.2.3"
'
    assert equal (read_ws_version $toml) "1.2.3"
}

def "test version: reads pre-release version" [] {
    let toml = '[workspace.package]
version = "0.5.0-rc.1"
'
    assert equal (read_ws_version $toml) "0.5.0-rc.1"
}

def "test version: script output matches Cargo.toml" [] {
    let root = ($env.CURRENT_FILE | path dirname | path dirname | path dirname)
    let out = (nu ($root | path join scripts version.nu) | str trim)
    let expected = (open ($root | path join Cargo.toml) | get workspace.package.version)
    assert equal $out $expected
}

def main [] { run-tests }
