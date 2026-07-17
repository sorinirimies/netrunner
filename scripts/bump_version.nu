#!/usr/bin/env nu
# Bump the package version for netrunner_cli.
# Updates Cargo.toml, README.md badge, Cargo.lock, runs checks, generates
# CHANGELOG, commits, and creates an annotated git tag.
#
# Usage:  nu scripts/bump_version.nu <new_version>
# Example: nu scripts/bump_version.nu 0.7.0

def main [
    new_version: string,  # Target version (e.g. 0.7.0 or 1.0.0-beta.1)
    --skip-checks,        # Skip cargo clippy + test (caller already ran them)
] {
    let red    = (ansi red)
    let green  = (ansi green)
    let yellow = (ansi yellow)
    let cyan   = (ansi cyan)
    let reset  = (ansi reset)

    # ── Validate version format ───────────────────────────────────────────────
    let version_re = '^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$'
    if not ($new_version =~ $version_re) {
        print $"($red)Error: invalid version format '($new_version)'($reset)"
        print "Version must be X.Y.Z or X.Y.Z-pre (e.g. 0.7.0, 1.0.0-beta.1)"
        exit 1
    }

    # ── Read current version ──────────────────────────────────────────────────
    let current_version = (open Cargo.toml | get workspace.package.version)
    let tag_name = $"v($new_version)"

    print $"($cyan)netrunner_cli version bump($reset)"
    print $"  Current : ($yellow)($current_version)($reset)"
    print $"  New     : ($green)($new_version)($reset)"
    print ""

    # ── Guard: already at this version ───────────────────────────────────────
    if $current_version == $new_version {
        print $"($yellow)Already at version ($new_version) — nothing to do.($reset)"
        exit 0
    }

    # ── Guard: local tag already exists ──────────────────────────────────────
    let tag_exists = (
        do { run-external "git" "tag" "--list" $tag_name } | str trim | is-not-empty
    )
    if $tag_exists {
        print $"($red)Error: tag ($tag_name) already exists locally.($reset)"
        print $"Delete it first with:  git tag -d ($tag_name)"
        exit 1
    }

    print ""

    # ─────────────────────────────────────────────────────────────────────────
    # Step 1/8 — Update Cargo.toml
    # ─────────────────────────────────────────────────────────────────────────
    print $"($cyan)[1/8]($reset) Updating Cargo.toml …"
    # Bump both the [workspace.package] version (first `^version = "..."`) and
    # the internal `netrunner-core` path-dependency pin so member crates keep
    # resolving each other. Inline third-party `{ version = "..." }` entries are
    # left untouched (they don't start a line).
    let cargo_toml = (open --raw Cargo.toml
        | str replace --regex '(?m)^version = "[^"]*"' $"version = \"($new_version)\""
        | str replace --all --regex 'netrunner-core = \{ path = "crates/netrunner-core", version = "[^"]*" \}' $"netrunner-core = { path = \"crates/netrunner-core\", version = \"($new_version)\" }")
    $cargo_toml | save --force Cargo.toml
    print $"  ($green)✔ Cargo.toml updated($reset)"

    # ─────────────────────────────────────────────────────────────────────────
    # Step 2/8 — Update README.md version badge
    # ─────────────────────────────────────────────────────────────────────────
    print $"($cyan)[2/8]($reset) Updating README.md badge …"
    let readme = (open --raw README.md
        | str replace --all --regex 'version-[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?-blue' $"version-($new_version)-blue")
    $readme | save --force README.md
    print $"  ($green)✔ README.md badge updated($reset)"

    # ─────────────────────────────────────────────────────────────────────────
    # Step 3/8 — Regenerate Cargo.lock
    # ─────────────────────────────────────────────────────────────────────────
    print $"($cyan)[3/8]($reset) Regenerating Cargo.lock …"
    run-external "cargo" "generate-lockfile"
    print $"  ($green)✔ Cargo.lock updated($reset)"

    # ─────────────────────────────────────────────────────────────────────────
    # Step 4/8 — cargo fmt
    # ─────────────────────────────────────────────────────────────────────────
    print $"($cyan)[4/8]($reset) Running cargo fmt …"
    run-external "cargo" "fmt"
    print $"  ($green)✔ Formatting applied($reset)"

    # ─────────────────────────────────────────────────────────────────────────
    # Step 5/8 — cargo clippy (skippable — the caller may have run it already)
    # ─────────────────────────────────────────────────────────────────────────
    if $skip_checks {
        print $"($cyan)[5/8]($reset) Skipping cargo clippy \(--skip-checks\) …"
    } else {
        print $"($cyan)[5/8]($reset) Running cargo clippy …"
        let clippy = (
            do { run-external "cargo" "clippy" "--all-targets" "--all-features" "--" "-D" "warnings" "-A" "deprecated" }
            | complete
        )
        if $clippy.exit_code != 0 {
            print $"  ($red)✘ Clippy reported errors \(exit ($clippy.exit_code)\).($reset)"
            print "  Fix the issues above and re-run the script."
            exit 1
        }
        print $"  ($green)✔ Clippy passed($reset)"
    }

    # ─────────────────────────────────────────────────────────────────────────
    # Step 6/8 — cargo test (skippable — the caller may have run it already)
    # ─────────────────────────────────────────────────────────────────────────
    if $skip_checks {
        print $"($cyan)[6/8]($reset) Skipping cargo test \(--skip-checks\) …"
    } else {
        print $"($cyan)[6/8]($reset) Running cargo test …"
        let tests = (
            do { run-external "cargo" "test" "--all-features" "--all-targets" }
            | complete
        )
        if $tests.exit_code != 0 {
            print $"  ($red)✘ Tests failed \(exit ($tests.exit_code)\).($reset)"
            print "  Fix the failures above and re-run the script."
            exit 1
        }
        print $"  ($green)✔ All tests passed($reset)"
    }

    # ─────────────────────────────────────────────────────────────────────────
    # Step 7/8 — CHANGELOG, commit, tag
    # ─────────────────────────────────────────────────────────────────────────
    print $"($cyan)[7/8]($reset) Generating CHANGELOG, committing, and tagging …"

    # Generate CHANGELOG.md with git-cliff
    let cliff = (
        do { run-external "git-cliff" "--tag" $tag_name "-o" "CHANGELOG.md" }
        | complete
    )
    if $cliff.exit_code != 0 {
        print $"  ($yellow)⚠ git-cliff exited ($cliff.exit_code) — continuing without updated CHANGELOG.($reset)"
        print "  Install git-cliff with: cargo install git-cliff"
    } else {
        print $"  ($green)✔ CHANGELOG.md generated($reset)"
    }

    # Stage files
    run-external "git" "add" "Cargo.toml" "Cargo.lock" "README.md" "CHANGELOG.md"

    # Commit
    run-external "git" "commit" "-m" $"chore: bump version to ($new_version)"
    print $"  ($green)✔ Commit created($reset)"

    # Annotated tag
    run-external "git" "tag" "-a" $tag_name "-m" $"Release ($tag_name)"
    print $"  ($green)✔ Tag ($tag_name) created($reset)"

    # ─────────────────────────────────────────────────────────────────────────
    # Step 8 — Push commits and tags
    # ─────────────────────────────────────────────────────────────────────────
    print $"($cyan)[8/8]($reset) Pushing commits and tags …"

    # Push to origin (GitHub)
    let push_main = (
        do { run-external "git" "push" "origin" "main" }
        | complete
    )
    if $push_main.exit_code != 0 {
        print $"  ($yellow)⚠ Failed to push to origin/main \(exit ($push_main.exit_code)\)($reset)"
    } else {
        print $"  ($green)✔ Pushed to origin/main($reset)"
    }

    let push_tag_origin = (
        do { run-external "git" "push" "origin" $tag_name }
        | complete
    )
    if $push_tag_origin.exit_code != 0 {
        print $"  ($yellow)⚠ Failed to push tag ($tag_name) to origin \(exit ($push_tag_origin.exit_code)\)($reset)"
    } else {
        print $"  ($green)✔ Pushed tag ($tag_name) to origin($reset)"
    }

    # Push to gitea (if remote exists)
    let gitea_remote = (
        do { run-external "git" "remote" "get-url" "gitea" }
        | complete
    )
    if $gitea_remote.exit_code == 0 {
        let push_gitea = (
            do { run-external "git" "push" "gitea" "main" }
            | complete
        )
        if $push_gitea.exit_code != 0 {
            print $"  ($yellow)⚠ Failed to push to gitea/main \(exit ($push_gitea.exit_code)\)($reset)"
        } else {
            print $"  ($green)✔ Pushed to gitea/main($reset)"
        }

        let push_tag_gitea = (
            do { run-external "git" "push" "gitea" $tag_name }
            | complete
        )
        if $push_tag_gitea.exit_code != 0 {
            print $"  ($yellow)⚠ Failed to push tag ($tag_name) to gitea \(exit ($push_tag_gitea.exit_code)\)($reset)"
        } else {
            print $"  ($green)✔ Pushed tag ($tag_name) to gitea($reset)"
        }
    } else {
        print $"  ($yellow)⚠ gitea remote not configured — skipping($reset)"
    }

    # ─────────────────────────────────────────────────────────────────────────
    # Summary
    # ─────────────────────────────────────────────────────────────────────────
    print ""
    print $"($green)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"
    print $"($green)  Version ($new_version) released! 🚀($reset)"
    print $"($green)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━($reset)"
    print ""
    print $"  The CI release workflow will now:"
    print $"    • Build the release binary"
    print $"    • Create a GitHub/Gitea release"
    print $"    • Publish to crates.io"
    print ""
}
