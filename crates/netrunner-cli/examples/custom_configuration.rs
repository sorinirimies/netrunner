//! # Custom Configuration Example
//!
//! This example demonstrates how to customize NetRunner's speed test configuration
//! for different use cases and network conditions.
//!
//! Usage:
//!   cargo run --example custom_configuration
//!
//! Features demonstrated:
//! - Creating custom test configurations
//! - Adjusting test parameters for different scenarios
//! - Timeout configuration
//! - Server selection options
//! - Output format options

use netrunner_cli::{DetailLevel, SpeedTest, TestConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install crypto provider");

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║      NetRunner CLI - Custom Configuration Example        ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    // Configuration 1: Quick Test (for fast checks)
    println!("═══════════════════════════════════════════════════════════");
    println!("Configuration 1: Quick Test");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("Use case: Quick network check, minimal time");
    println!();

    let quick_config = TestConfig {
        server_url: "https://speed.cloudflare.com".to_string(),
        test_size_mb: 25,    // Small test size
        timeout_seconds: 30, // Short timeout
        json_output: false,
        animation_enabled: false, // Faster without animations
        detail_level: DetailLevel::Standard,
        max_servers: 1, // Test only 1 server
    };

    println!("Configuration:");
    println!("  • Test Size: {} MB (small)", quick_config.test_size_mb);
    println!("  • Timeout: {} seconds", quick_config.timeout_seconds);
    println!("  • Max Servers: {} (fastest)", quick_config.max_servers);
    println!("  • Animations: Disabled");
    println!("  • Expected Duration: ~20-30 seconds");
    println!();

    // Configuration 2: Accurate Test (for detailed measurements)
    println!("═══════════════════════════════════════════════════════════");
    println!("Configuration 2: Accurate Test");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("Use case: Detailed analysis, maximum accuracy");
    println!();

    let accurate_config = TestConfig {
        server_url: "https://speed.cloudflare.com".to_string(),
        test_size_mb: 200,    // Larger test size
        timeout_seconds: 120, // Longer timeout
        json_output: false,
        animation_enabled: true, // Full experience
        detail_level: DetailLevel::Debug,
        max_servers: 5, // Test multiple servers
    };

    println!("Configuration:");
    println!("  • Test Size: {} MB (large)", accurate_config.test_size_mb);
    println!("  • Timeout: {} seconds", accurate_config.timeout_seconds);
    println!(
        "  • Max Servers: {} (best accuracy)",
        accurate_config.max_servers
    );
    println!("  • Animations: Enabled");
    println!("  • Detail Level: Debug");
    println!("  • Expected Duration: ~60-90 seconds");
    println!();

    // Configuration 3: CI/CD Pipeline (for automation)
    println!("═══════════════════════════════════════════════════════════");
    println!("Configuration 3: CI/CD Pipeline");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("Use case: Automated testing, machine-readable output");
    println!();

    let cicd_config = TestConfig {
        server_url: "https://speed.cloudflare.com".to_string(),
        test_size_mb: 50,
        timeout_seconds: 60,
        json_output: true,        // JSON for parsing
        animation_enabled: false, // No UI in CI/CD
        detail_level: DetailLevel::Standard,
        max_servers: 2,
    };

    println!("Configuration:");
    println!("  • Test Size: {} MB", cicd_config.test_size_mb);
    println!("  • Timeout: {} seconds", cicd_config.timeout_seconds);
    println!("  • Max Servers: {}", cicd_config.max_servers);
    println!("  • JSON Output: Enabled (no UI)");
    println!("  • Perfect for: GitHub Actions, Jenkins, GitLab CI");
    println!();

    // Configuration 4: Slow Connection (for limited bandwidth)
    println!("═══════════════════════════════════════════════════════════");
    println!("Configuration 4: Slow Connection");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("Use case: Testing on slow or mobile connections");
    println!();

    let slow_config = TestConfig {
        server_url: "https://speed.cloudflare.com".to_string(),
        test_size_mb: 10,    // Very small test
        timeout_seconds: 90, // Longer timeout for slow speeds
        json_output: false,
        animation_enabled: true,
        detail_level: DetailLevel::Standard,
        max_servers: 1,
    };

    println!("Configuration:");
    println!("  • Test Size: {} MB (minimal)", slow_config.test_size_mb);
    println!(
        "  • Timeout: {} seconds (patient)",
        slow_config.timeout_seconds
    );
    println!("  • Max Servers: {}", slow_config.max_servers);
    println!("  • Optimized for: DSL, Mobile, Satellite");
    println!();

    // Configuration 5: Gigabit Test (for high-speed connections)
    println!("═══════════════════════════════════════════════════════════");
    println!("Configuration 5: Gigabit Test");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("Use case: Testing fiber/gigabit connections");
    println!();

    let gigabit_config = TestConfig {
        server_url: "https://speed.cloudflare.com".to_string(),
        test_size_mb: 500, // Very large test
        timeout_seconds: 120,
        json_output: false,
        animation_enabled: true,
        detail_level: DetailLevel::Debug,
        max_servers: 3,
    };

    println!("Configuration:");
    println!(
        "  • Test Size: {} MB (maximum)",
        gigabit_config.test_size_mb
    );
    println!("  • Timeout: {} seconds", gigabit_config.timeout_seconds);
    println!("  • Max Servers: {}", gigabit_config.max_servers);
    println!("  • Optimized for: Fiber, Gigabit Ethernet, 10G connections");
    println!();

    // Now let's run one of these configurations
    println!("═══════════════════════════════════════════════════════════");
    println!("Running Quick Test Configuration");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    println!("🚀 Starting quick test...");
    let speed_test = SpeedTest::new(quick_config)?;

    match speed_test.run_full_test().await {
        Ok(result) => {
            println!();
            println!("✅ Quick Test Results:");
            println!("   ↓ Download: {:.2} Mbps", result.download_mbps);
            println!("   ↑ Upload:   {:.2} Mbps", result.upload_mbps);
            println!("   📡 Ping:    {:.2} ms", result.ping_ms);
            println!("   ⚡ Quality:  {:?}", result.quality);
            println!();

            // Provide recommendations based on results
            println!("💡 Recommendations:");
            println!();

            if result.download_mbps < 25.0 {
                println!("   Your connection appears slow. Consider:");
                println!("   • Use 'slow_config' for more accurate results");
                println!("   • Check for background downloads");
                println!("   • Contact your ISP if speeds are consistently low");
            } else if result.download_mbps > 500.0 {
                println!("   You have a fast connection! Consider:");
                println!("   • Use 'gigabit_config' for full bandwidth testing");
                println!("   • Increase test_size_mb for better accuracy");
                println!("   • Test with max_servers: 5 for comprehensive results");
            } else {
                println!("   Your connection is typical. Consider:");
                println!("   • Use 'accurate_config' for detailed measurements");
                println!("   • Run tests at different times of day");
                println!("   • Compare with historical results");
            }
        }
        Err(e) => {
            println!("❌ Test failed: {}", e);
            println!();
            println!("💡 Troubleshooting tips:");
            println!("   • Increase timeout_seconds for slow connections");
            println!("   • Reduce test_size_mb if timeouts occur");
            println!("   • Check firewall/proxy settings");
            println!("   • Try a different server_url");
        }
    }

    println!();
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║              Configuration Examples Summary               ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    println!("📋 Choose the right configuration for your use case:");
    println!();
    println!("  1. Quick Test      → Fast checks, minimal time");
    println!("  2. Accurate Test   → Detailed analysis, best accuracy");
    println!("  3. CI/CD Pipeline  → Automation, JSON output");
    println!("  4. Slow Connection → Mobile, DSL, satellite");
    println!("  5. Gigabit Test    → Fiber, high-speed connections");
    println!();
    println!("💡 Tips for customization:");
    println!();
    println!("  • test_size_mb: Larger = more accurate, longer test");
    println!("  • timeout_seconds: Adjust based on expected speeds");
    println!("  • max_servers: More servers = better selection, longer init");
    println!("  • json_output: Enable for automation/parsing");
    println!("  • animation_enabled: Disable for faster tests");
    println!("  • detail_level: Debug for troubleshooting");
    println!();
    println!("📖 See other examples for more features:");
    println!("  • basic_speed_test.rs - Simple usage");
    println!("  • json_output.rs - JSON integration");
    println!("  • history_management.rs - Track results over time");

    Ok(())
}
