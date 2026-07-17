#!/usr/bin/env nu
# Shared test runner for netrunner's Nu script tests.
#
# Usage in a test file:
#   use runner.nu *
#   def "test <name>" [] { ... }
#   def main [] { run-tests }
#
# Discovers every custom command named "test *" in the calling file, runs each
# in isolation, prints a summary, and exits non-zero if any test fails.

export def run-tests [] {
    let green = (ansi green)
    let red = (ansi red)
    let bold = (ansi --escape {attr: b})
    let cyan = (ansi cyan)
    let reset = (ansi reset)

    let tests = (
        scope commands
        | where { |it| $it.type == "custom" }
        | where { |it| $it.name | str starts-with "test " }
        | where { |it| not ($it.description | str starts-with "ignore") }
        | get name
        | sort
    )

    let total = ($tests | length)

    if $total == 0 {
        print $"($red)No tests found.($reset)"
        exit 1
    }

    mut passed = 0
    mut failed = 0
    mut failures = []

    for test in $tests {
        print -n $"  ($cyan)($test)($reset) ... "
        let res = (do { nu --commands $"source '($env.CURRENT_FILE)'; ($test)" } | complete)

        if $res.exit_code == 0 {
            print $"($green)ok($reset)"
            $passed = $passed + 1
        } else {
            print $"($red)FAILED($reset)"
            $failed = $failed + 1
            $failures = ($failures | append { test: $test, output: $res.stderr })
        }
    }

    print ""
    print $"Results: ($green)($passed) passed($reset) · ($red)($failed) failed($reset) · ($total) total"

    if $failed > 0 {
        print ""
        print $"($red)($bold)Failures:($reset)"
        for f in $failures {
            print $"  ($bold)($f.test)($reset)"
            $f.output | lines | each { |l| print $"    ($l)" }
            print ""
        }
        exit 1
    }

    print ""
    print $"($green)✓ All ($total) tests passed!($reset)"
}
