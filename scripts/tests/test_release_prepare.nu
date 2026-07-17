#!/usr/bin/env nu
# Tests for scripts/release_prepare.nu — the tag → bare-version transform used
# to derive the crate version from a pushed git tag.

use std/assert
use runner.nu *

# Mirrors: let version = ($tag | str replace --regex '^v' '')
def strip_v [tag: string]: nothing -> string {
    $tag | str replace --regex '^v' ''
}

def "test release: strips leading v" [] {
    assert equal (strip_v "v1.2.3") "1.2.3"
    assert equal (strip_v "v2.0.0-rc.1") "2.0.0-rc.1"
}

def "test release: leaves a bare version unchanged" [] {
    assert equal (strip_v "1.2.3") "1.2.3"
}

def "test release: only strips the first leading v" [] {
    # A 'v' inside the version must not be touched (there shouldn't be one, but
    # guard the anchored regex behaviour).
    assert equal (strip_v "v1.2.3-vv") "1.2.3-vv"
}

def main [] { run-tests }
