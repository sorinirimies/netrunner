//! Binary and package weight tests
//!
//! These tests verify that the published crate stays within crates.io limits
//! and doesn't accidentally include large assets or pull in heavy dependencies.

use std::process::Command;

/// Verify that the package exclude patterns are properly configured
/// by checking that known large files are not in the expected package list.
#[test]
fn test_excluded_files_not_in_package() {
    let output = Command::new("cargo")
        .args(["package", "--list", "--allow-dirty"])
        .output()
        .expect("Failed to run cargo package --list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files: Vec<&str> = stdout.lines().collect();

    // GIFs must be excluded
    for file in &files {
        assert!(
            !file.ends_with(".gif"),
            "GIF file should be excluded from package: {file}"
        );
    }

    // CI workflow files must be excluded
    for file in &files {
        assert!(
            !file.starts_with(".github/") && !file.starts_with(".gitea/"),
            "CI workflow files should be excluded from package: {file}"
        );
    }

    // Scripts must be excluded
    for file in &files {
        assert!(
            !file.starts_with("scripts/"),
            "Scripts should be excluded from package: {file}"
        );
    }

    // VHS tape files must be excluded
    for file in &files {
        assert!(
            !file.starts_with("examples/vhs/"),
            "VHS files should be excluded from package: {file}"
        );
    }

    // justfile and cliff.toml must be excluded
    for file in &files {
        assert!(
            *file != "justfile" && *file != "cliff.toml" && *file != "results.json",
            "Build tool config should be excluded from package: {file}"
        );
    }
}

/// Verify that the total number of packaged files stays reasonable.
/// If this test fails, someone likely added a large directory that
/// should be in the exclude list.
#[test]
fn test_package_file_count_reasonable() {
    let output = Command::new("cargo")
        .args(["package", "--list", "--allow-dirty"])
        .output()
        .expect("Failed to run cargo package --list");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let file_count = stdout.lines().count();

    // We expect roughly 30-40 files. Fail if it balloons past 50.
    assert!(
        file_count <= 50,
        "Package contains {file_count} files — expected ≤ 50. \
         Check if new directories need to be added to the exclude list in Cargo.toml."
    );
}

/// Verify that Cargo.toml does not depend on known heavy crates
/// that we have intentionally removed.
#[test]
fn test_no_heavy_banned_direct_dependencies() {
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    // sled was replaced by redb
    assert!(
        !cargo_toml.contains("sled"),
        "Cargo.toml should not depend on sled — use redb instead"
    );
}

/// Verify that the dependency tree does not contain aws-lc-sys
/// (we use ring as the crypto provider instead).
#[test]
fn test_no_aws_lc_sys_in_dependency_tree() {
    let output = Command::new("cargo")
        .args(["tree", "--prefix", "none", "--no-dedupe"])
        .output()
        .expect("Failed to run cargo tree");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.contains("aws-lc-sys"),
        "Dependency tree should not contain aws-lc-sys — \
         ensure reqwest uses ring instead of aws-lc-rs as the crypto provider"
    );
}

/// Verify that the Cargo.toml exclude list contains expected patterns.
#[test]
fn test_cargo_toml_has_exclude_patterns() {
    let cargo_toml = std::fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    // After the workspace split, CI/scripts live at the repo root and are not
    // part of this crate's package at all. The crate only needs to exclude its
    // own bulky assets.
    let expected_patterns = ["examples/vhs/", "results.json"];

    for pattern in &expected_patterns {
        assert!(
            cargo_toml.contains(pattern),
            "Cargo.toml exclude list is missing pattern: {pattern}"
        );
    }
}

/// Verify that the workspace release profile has size optimization settings.
#[test]
fn test_release_profile_optimized() {
    // The release profile is defined once at the workspace root.
    let cargo_toml =
        std::fs::read_to_string("../../Cargo.toml").expect("Failed to read workspace Cargo.toml");

    assert!(
        cargo_toml.contains("[profile.release]"),
        "Workspace Cargo.toml should have a [profile.release] section"
    );
    assert!(
        cargo_toml.contains("strip = true"),
        "Release profile should strip symbols"
    );
    assert!(
        cargo_toml.contains("lto = true"),
        "Release profile should enable LTO"
    );
}
