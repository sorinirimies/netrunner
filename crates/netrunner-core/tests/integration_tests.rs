use netrunner_core::{
    speed_test::SpeedTest,
    types::{ConnectionQuality, DetailLevel, TestConfig},
};
use std::time::Duration;

fn ensure_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

#[tokio::test]
async fn test_speed_test_creation() {
    ensure_crypto_provider();
    let config = TestConfig::default();
    let speed_test = SpeedTest::new(config);
    assert!(speed_test.is_ok());
}

#[tokio::test]
async fn test_speed_test_with_custom_config() {
    ensure_crypto_provider();
    let config = TestConfig {
        server_url: "https://httpbin.org".to_string(),
        test_size_mb: 5,
        timeout_seconds: 10,
        json_output: true,
        animation_enabled: false,
        detail_level: DetailLevel::Basic,
        max_servers: 1,
    };

    let speed_test = SpeedTest::new(config);
    assert!(speed_test.is_ok());
}

#[tokio::test]
async fn test_connection_quality_rating() {
    ensure_crypto_provider();
    // Test excellent connection
    let quality = ConnectionQuality::from_speed_and_ping(150.0, 30.0, 10.0);
    assert_eq!(quality, ConnectionQuality::Excellent);

    // Test good connection
    let quality = ConnectionQuality::from_speed_and_ping(75.0, 15.0, 30.0);
    assert_eq!(quality, ConnectionQuality::Good);

    // Test average connection
    let quality = ConnectionQuality::from_speed_and_ping(30.0, 8.0, 80.0);
    assert_eq!(quality, ConnectionQuality::Average);

    // Test poor connection
    let quality = ConnectionQuality::from_speed_and_ping(15.0, 3.0, 120.0);
    assert_eq!(quality, ConnectionQuality::Poor);

    // Test very poor connection
    let quality = ConnectionQuality::from_speed_and_ping(5.0, 1.0, 200.0);
    assert_eq!(quality, ConnectionQuality::VeryPoor);

    // Test failed connection
    let quality = ConnectionQuality::from_speed_and_ping(0.0, 0.0, 0.0);
    assert_eq!(quality, ConnectionQuality::Failed);
}

#[tokio::test]
async fn test_speed_test_timeout() {
    ensure_crypto_provider();
    let config = TestConfig {
        server_url: "https://httpbin.org/delay/20".to_string(), // This will timeout
        test_size_mb: 1,
        timeout_seconds: 1, // Very short timeout
        json_output: true,
        animation_enabled: false,
        detail_level: DetailLevel::Basic,
        max_servers: 1,
    };

    let speed_test = SpeedTest::new(config).unwrap();
    let result = speed_test.run_full_test().await;

    // Should still return a result even if some tests fail
    assert!(result.is_ok());
    let test_result = result.unwrap();
    assert!(test_result.download_mbps >= 0.0);
    assert!(test_result.upload_mbps >= 0.0);
    assert!(test_result.ping_ms >= 0.0);
}

#[tokio::test]
async fn test_speed_test_result_structure() {
    ensure_crypto_provider();
    let config = TestConfig {
        server_url: "https://httpbin.org".to_string(),
        test_size_mb: 1,
        timeout_seconds: 30,
        json_output: true,
        animation_enabled: false,
        detail_level: DetailLevel::Standard,
        max_servers: 1,
    };

    let speed_test = SpeedTest::new(config).unwrap();
    let result = speed_test.run_full_test().await;

    assert!(result.is_ok());
    let test_result = result.unwrap();

    // Verify result structure
    assert!(test_result.download_mbps >= 0.0);
    assert!(test_result.upload_mbps >= 0.0);
    assert!(test_result.ping_ms >= 0.0);
    assert!(test_result.test_duration_seconds > 0.0);
    assert!(!test_result.server_location.is_empty());

    // Verify timestamp is recent (within last minute)
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(test_result.timestamp);
    assert!(diff.num_seconds() < 60);
}

#[tokio::test]
async fn test_multiple_speed_tests() {
    ensure_crypto_provider();
    let config = TestConfig {
        server_url: "https://httpbin.org".to_string(),
        test_size_mb: 1,
        timeout_seconds: 15,
        json_output: true,
        animation_enabled: false,
        detail_level: DetailLevel::Basic,
        max_servers: 1,
    };

    // Run multiple tests to ensure consistency
    for _ in 0..3 {
        let speed_test = SpeedTest::new(config.clone()).unwrap();
        let result = speed_test.run_full_test().await;
        assert!(result.is_ok());

        let test_result = result.unwrap();
        assert!(test_result.download_mbps >= 0.0);
        assert!(test_result.upload_mbps >= 0.0);
        assert!(test_result.ping_ms >= 0.0);

        // Add small delay between tests
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
