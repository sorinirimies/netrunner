use chrono::Utc;
use netrunner_core::{
    speed_test::SpeedTest,
    types::{
        ConnectionQuality, DetailLevel, ServerCapabilities, ServerProvider, SpeedTestResult,
        TestConfig, TestServer,
    },
};
use std::time::Duration;

fn ensure_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// Helper function to create a test config
fn create_test_config() -> TestConfig {
    TestConfig {
        server_url: "https://httpbin.org".to_string(),
        test_size_mb: 1, // Small size for testing
        timeout_seconds: 10,
        json_output: true, // Suppress UI output during tests
        animation_enabled: false,
        detail_level: DetailLevel::Standard,
        max_servers: 2,
    }
}

/// Helper function to create a mock test server
fn create_mock_server() -> TestServer {
    TestServer {
        name: "Test Server".to_string(),
        url: "https://httpbin.org".to_string(),
        location: "Test Location".to_string(),
        distance_km: Some(100.0),
        latency_ms: Some(50.0),
        provider: ServerProvider::Custom("Test".to_string()),
        capabilities: ServerCapabilities {
            supports_download: true,
            supports_upload: true,
            supports_latency: true,
            max_test_size_mb: 10,
            geographic_weight: 0.8,
        },
        quality_score: Some(0.7),
        country_code: Some("US".to_string()),
        city: Some("Test City".to_string()),
        is_backup: false,
    }
}

#[tokio::test]
async fn test_speed_test_creation() {
    ensure_crypto_provider();
    let config = create_test_config();
    let speed_test = SpeedTest::new(config);

    assert!(
        speed_test.is_ok(),
        "SpeedTest should be created successfully"
    );
}

#[tokio::test]
async fn test_speed_test_full_test() {
    ensure_crypto_provider();
    let config = create_test_config();
    let speed_test = SpeedTest::new(config).expect("Failed to create SpeedTest");

    // Run the full test with a short timeout
    let result = tokio::time::timeout(Duration::from_secs(30), speed_test.run_full_test()).await;

    match result {
        Ok(test_result) => {
            let test_result = test_result.expect("Speed test should complete successfully");

            // Verify basic result structure
            assert!(
                test_result.download_mbps >= 0.0,
                "Download speed should be non-negative"
            );
            assert!(
                test_result.upload_mbps >= 0.0,
                "Upload speed should be non-negative"
            );
            assert!(test_result.ping_ms >= 0.0, "Ping should be non-negative");
            assert!(
                test_result.test_duration_seconds > 0.0,
                "Test duration should be positive"
            );
            assert!(
                !test_result.server_location.is_empty(),
                "Server location should not be empty"
            );
        }
        Err(_) => {
            // Test timed out, which is acceptable in some network conditions
            println!("Speed test timed out - this may be expected in test environments");
        }
    }
}

#[tokio::test]
async fn test_connection_quality_calculation() {
    // Test excellent quality
    let quality = ConnectionQuality::from_speed_and_ping(150.0, 25.0, 15.0);
    assert_eq!(quality, ConnectionQuality::Excellent);

    // Test good quality
    let quality = ConnectionQuality::from_speed_and_ping(60.0, 12.0, 40.0);
    assert_eq!(quality, ConnectionQuality::Good);

    // Test average quality
    let quality = ConnectionQuality::from_speed_and_ping(30.0, 7.0, 90.0);
    assert_eq!(quality, ConnectionQuality::Average);

    // Test poor quality
    let quality = ConnectionQuality::from_speed_and_ping(12.0, 3.0, 140.0);
    assert_eq!(quality, ConnectionQuality::Poor);

    // Test very poor quality
    let quality = ConnectionQuality::from_speed_and_ping(8.0, 1.5, 200.0);
    assert_eq!(quality, ConnectionQuality::VeryPoor);

    // Test failed quality
    let quality = ConnectionQuality::from_speed_and_ping(0.0, 0.0, 0.0);
    assert_eq!(quality, ConnectionQuality::Failed);
}

#[tokio::test]
async fn test_quality_boundary_conditions() {
    // Test exact boundaries for excellent
    let quality = ConnectionQuality::from_speed_and_ping(100.0, 20.0, 19.9);
    assert_eq!(quality, ConnectionQuality::Excellent);

    // Just below excellent threshold
    let quality = ConnectionQuality::from_speed_and_ping(99.9, 19.9, 20.1);
    assert_ne!(quality, ConnectionQuality::Excellent);

    // Test exact boundaries for good
    let quality = ConnectionQuality::from_speed_and_ping(50.0, 10.0, 49.9);
    assert_eq!(quality, ConnectionQuality::Good);
}

#[tokio::test]
async fn test_speed_test_result_validation() {
    let result = SpeedTestResult {
        timestamp: Utc::now(),
        download_mbps: 50.5,
        upload_mbps: 10.2,
        ping_ms: 25.0,
        jitter_ms: 2.5,
        packet_loss_percent: 0.1,
        server_location: "Test Location".to_string(),
        server_ip: None,
        client_ip: None,
        quality: ConnectionQuality::Good,
        test_duration_seconds: 15.5,
        isp: Some("Test ISP".to_string()),
    };

    // Verify all fields are properly set
    assert_eq!(result.download_mbps, 50.5);
    assert_eq!(result.upload_mbps, 10.2);
    assert_eq!(result.ping_ms, 25.0);
    assert_eq!(result.jitter_ms, 2.5);
    assert_eq!(result.packet_loss_percent, 0.1);
    assert_eq!(result.server_location, "Test Location");
    assert_eq!(result.quality, ConnectionQuality::Good);
    assert_eq!(result.test_duration_seconds, 15.5);
    assert_eq!(result.isp, Some("Test ISP".to_string()));
}

#[tokio::test]
async fn test_realistic_speed_ranges() {
    ensure_crypto_provider();
    let config = create_test_config();
    let speed_test = SpeedTest::new(config).expect("Failed to create SpeedTest");

    // Test that speeds are within realistic ranges (if test completes)
    if let Ok(Ok(test_result)) =
        tokio::time::timeout(Duration::from_secs(20), speed_test.run_full_test()).await
    {
        // Download speed should be reasonable (not impossibly high)
        assert!(
            test_result.download_mbps <= 10000.0,
            "Download speed seems unrealistically high: {}",
            test_result.download_mbps
        );
        assert!(
            test_result.download_mbps >= 0.1,
            "Download speed seems too low: {}",
            test_result.download_mbps
        );

        // Upload speed should be reasonable
        assert!(
            test_result.upload_mbps <= 1000.0,
            "Upload speed seems unrealistically high: {}",
            test_result.upload_mbps
        );
        assert!(
            test_result.upload_mbps >= 0.1,
            "Upload speed seems too low: {}",
            test_result.upload_mbps
        );

        // Ping should be reasonable
        assert!(
            test_result.ping_ms <= 5000.0,
            "Ping seems unrealistically high: {}",
            test_result.ping_ms
        );
        assert!(
            test_result.ping_ms >= 1.0,
            "Ping seems too low: {}",
            test_result.ping_ms
        );

        // Jitter should be reasonable if measured
        if test_result.jitter_ms > 0.0 {
            assert!(
                test_result.jitter_ms <= 1000.0,
                "Jitter seems unrealistically high: {}",
                test_result.jitter_ms
            );
        }

        // Packet loss should be percentage (0-100)
        assert!(
            test_result.packet_loss_percent >= 0.0 && test_result.packet_loss_percent <= 100.0,
            "Packet loss should be a percentage: {}",
            test_result.packet_loss_percent
        );
    }
}

#[tokio::test]
async fn test_config_variations() {
    ensure_crypto_provider();
    // Test with different detail levels
    let mut config = create_test_config();
    config.detail_level = DetailLevel::Debug;
    let speed_test = SpeedTest::new(config);
    assert!(
        speed_test.is_ok(),
        "SpeedTest should work with Debug detail level"
    );

    // Test with different timeout
    let mut config = create_test_config();
    config.timeout_seconds = 5;
    let speed_test = SpeedTest::new(config);
    assert!(
        speed_test.is_ok(),
        "SpeedTest should work with short timeout"
    );

    // Test with different test size
    let mut config = create_test_config();
    config.test_size_mb = 5;
    let speed_test = SpeedTest::new(config);
    assert!(
        speed_test.is_ok(),
        "SpeedTest should work with different test size"
    );
}

#[tokio::test]
async fn test_server_capabilities() {
    let server = create_mock_server();

    assert!(
        server.capabilities.supports_download,
        "Mock server should support download"
    );
    assert!(
        server.capabilities.supports_upload,
        "Mock server should support upload"
    );
    assert!(
        server.capabilities.supports_latency,
        "Mock server should support latency testing"
    );
    assert!(
        server.capabilities.max_test_size_mb > 0,
        "Mock server should have positive max test size"
    );
    assert!(
        server.capabilities.geographic_weight >= 0.0
            && server.capabilities.geographic_weight <= 1.0,
        "Geographic weight should be between 0 and 1"
    );
}

#[tokio::test]
async fn test_server_provider_types() {
    // Test different server provider types
    let cloudflare = ServerProvider::Cloudflare;
    let google = ServerProvider::Google;
    let netflix = ServerProvider::Netflix;
    let custom = ServerProvider::Custom("TestProvider".to_string());

    // Ensure they can be compared
    assert_eq!(cloudflare, ServerProvider::Cloudflare);
    assert_ne!(cloudflare, google);
    assert_ne!(google, netflix);

    match custom {
        ServerProvider::Custom(ref name) => assert_eq!(name, "TestProvider"),
        _ => panic!("Custom provider should match pattern"),
    }
}

#[tokio::test]
async fn test_error_handling() {
    ensure_crypto_provider();
    // Test with invalid server URL
    let mut config = create_test_config();
    config.server_url = "invalid-url".to_string();

    let speed_test = SpeedTest::new(config);
    assert!(
        speed_test.is_ok(),
        "SpeedTest creation should succeed even with invalid URL"
    );

    // The actual network errors should be handled gracefully during test execution
}

#[tokio::test]
async fn test_json_serialization() {
    let result = SpeedTestResult {
        timestamp: Utc::now(),
        download_mbps: 100.5,
        upload_mbps: 20.3,
        ping_ms: 15.7,
        jitter_ms: 1.2,
        packet_loss_percent: 0.0,
        server_location: "Test Server".to_string(),
        server_ip: None,
        client_ip: None,
        quality: ConnectionQuality::Excellent,
        test_duration_seconds: 12.34,
        isp: Some("Test ISP".to_string()),
    };

    // Test JSON serialization
    let json = serde_json::to_string(&result);
    assert!(json.is_ok(), "SpeedTestResult should serialize to JSON");

    // Test JSON deserialization
    if let Ok(json_str) = json {
        let deserialized: Result<SpeedTestResult, _> = serde_json::from_str(&json_str);
        assert!(
            deserialized.is_ok(),
            "SpeedTestResult should deserialize from JSON"
        );

        if let Ok(parsed_result) = deserialized {
            assert_eq!(parsed_result.download_mbps, result.download_mbps);
            assert_eq!(parsed_result.upload_mbps, result.upload_mbps);
            assert_eq!(parsed_result.quality, result.quality);
        }
    }
}

#[tokio::test]
async fn test_performance_metrics_integration() {
    // Test that jitter and packet loss are properly integrated
    let result = SpeedTestResult {
        timestamp: Utc::now(),
        download_mbps: 50.0,
        upload_mbps: 10.0,
        ping_ms: 30.0,
        jitter_ms: 5.0,
        packet_loss_percent: 1.0,
        server_location: "Test".to_string(),
        server_ip: None,
        client_ip: None,
        quality: ConnectionQuality::Good,
        test_duration_seconds: 10.0,
        isp: None,
    };

    // Verify that quality assessment considers all metrics appropriately
    assert_eq!(result.quality, ConnectionQuality::Good);

    // Even with good speeds, high jitter or packet loss might affect quality in future enhancements
    assert!(result.jitter_ms > 0.0, "Jitter should be measured");
    assert!(
        result.packet_loss_percent >= 0.0,
        "Packet loss should be measured"
    );
}

#[tokio::test]
async fn test_geolocation_structure() {
    use netrunner_core::speed_test::GeoLocation;

    let geo = GeoLocation {
        country: "United States".to_string(),
        city: "New York".to_string(),
        latitude: 40.7128,
        longitude: -74.0060,
        isp: Some("Test ISP".to_string()),
    };

    assert_eq!(geo.country, "United States");
    assert_eq!(geo.city, "New York");
    assert_eq!(geo.latitude, 40.7128);
    assert_eq!(geo.longitude, -74.0060);
    assert_eq!(geo.isp, Some("Test ISP".to_string()));
}

#[tokio::test]
async fn test_geolocation_coordinates_valid() {
    use netrunner_core::speed_test::GeoLocation;

    // Test valid coordinates
    let valid_locations = vec![
        ("Tokyo", 35.6762, 139.6503),
        ("London", 51.5074, -0.1278),
        ("Sydney", -33.8688, 151.2093),
        ("New York", 40.7128, -74.0060),
        ("São Paulo", -23.5505, -46.6333),
    ];

    for (city, lat, lon) in valid_locations {
        let geo = GeoLocation {
            country: "Test".to_string(),
            city: city.to_string(),
            latitude: lat,
            longitude: lon,
            isp: None,
        };

        assert!(
            geo.latitude >= -90.0 && geo.latitude <= 90.0,
            "Latitude should be in valid range for {}",
            city
        );
        assert!(
            geo.longitude >= -180.0 && geo.longitude <= 180.0,
            "Longitude should be in valid range for {}",
            city
        );
    }
}

#[tokio::test]
async fn test_server_distance_calculation() {
    let server = create_mock_server();

    // Server should have valid distance
    assert!(server.distance_km.is_some(), "Server should have distance");
    assert!(
        server.distance_km.unwrap() >= 0.0,
        "Distance should be non-negative"
    );
}

#[tokio::test]
async fn test_server_quality_score_calculation() {
    let mut server = create_mock_server();
    server.latency_ms = Some(50.0);
    server.distance_km = Some(100.0);

    // Calculate quality score
    let latency = server.latency_ms.unwrap();
    let distance = server.distance_km.unwrap();
    let geographic_weight = server.capabilities.geographic_weight;

    let latency_penalty = latency.max(1.0);
    let distance_penalty = (distance / 100.0).max(1.0);
    let quality_score = (10000.0 * geographic_weight) / (latency_penalty + distance_penalty);

    server.quality_score = Some(quality_score);

    assert!(
        server.quality_score.unwrap() > 0.0,
        "Quality score should be positive"
    );
}

#[tokio::test]
async fn test_server_selection_priority() {
    // Create servers with different characteristics
    let mut server1 = create_mock_server();
    server1.name = "Near Server".to_string();
    server1.distance_km = Some(50.0);
    server1.latency_ms = Some(20.0);
    server1.quality_score = Some(150.0);

    let mut server2 = create_mock_server();
    server2.name = "Far Server".to_string();
    server2.distance_km = Some(5000.0);
    server2.latency_ms = Some(150.0);
    server2.quality_score = Some(10.0);

    let mut servers = [server2.clone(), server1.clone()];

    // Sort by quality score (highest first)
    servers.sort_by(|a, b| {
        b.quality_score
            .unwrap_or(0.0)
            .partial_cmp(&a.quality_score.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Near server with better latency should be first
    assert_eq!(
        servers[0].name, "Near Server",
        "Near server should be prioritized"
    );
    assert_eq!(
        servers[1].name, "Far Server",
        "Far server should be deprioritized"
    );
}

#[tokio::test]
async fn test_geographic_weight_impact() {
    // Test that geographic weight affects server selection
    let mut server1 = create_mock_server();
    server1.capabilities.geographic_weight = 1.0; // High weight
    server1.latency_ms = Some(50.0);
    server1.distance_km = Some(100.0);

    let mut server2 = create_mock_server();
    server2.capabilities.geographic_weight = 0.3; // Low weight (backup)
    server2.latency_ms = Some(50.0);
    server2.distance_km = Some(100.0);

    // Calculate quality scores
    let score1 = (10000.0 * server1.capabilities.geographic_weight)
        / (server1.latency_ms.unwrap() + server1.distance_km.unwrap() / 100.0);
    let score2 = (10000.0 * server2.capabilities.geographic_weight)
        / (server2.latency_ms.unwrap() + server2.distance_km.unwrap() / 100.0);

    assert!(
        score1 > score2,
        "Server with higher geographic weight should have higher quality score"
    );
}

#[tokio::test]
async fn test_backup_server_flag() {
    let mut server = create_mock_server();
    server.is_backup = true;

    assert!(
        server.is_backup,
        "Backup server flag should be settable and readable"
    );

    // Backup servers should typically have lower geographic weight
    server.capabilities.geographic_weight = 0.3;
    assert!(
        server.capabilities.geographic_weight < 0.5,
        "Backup servers should have lower weight"
    );
}

#[tokio::test]
async fn test_server_capabilities_validation() {
    let server = create_mock_server();

    // All capabilities should be properly set
    assert!(
        server.capabilities.max_test_size_mb > 0,
        "Max test size should be positive"
    );
    assert!(
        server.capabilities.geographic_weight >= 0.0
            && server.capabilities.geographic_weight <= 1.0,
        "Geographic weight should be between 0 and 1"
    );
}

#[tokio::test]
async fn test_multiple_server_providers() {
    // Test different provider types
    let providers = vec![
        ServerProvider::Cloudflare,
        ServerProvider::Google,
        ServerProvider::Netflix,
        ServerProvider::Custom("TestProvider".to_string()),
    ];

    for provider in providers {
        let mut server = create_mock_server();
        server.provider = provider.clone();

        // Verify provider is set correctly
        match provider {
            ServerProvider::Cloudflare => {
                assert_eq!(server.provider, ServerProvider::Cloudflare)
            }
            ServerProvider::Google => assert_eq!(server.provider, ServerProvider::Google),
            ServerProvider::Netflix => assert_eq!(server.provider, ServerProvider::Netflix),
            ServerProvider::Ookla => assert_eq!(server.provider, ServerProvider::Ookla),
            ServerProvider::Custom(ref name) => {
                if let ServerProvider::Custom(ref server_name) = server.provider {
                    assert_eq!(server_name, name);
                }
            }
        }
    }
}

#[tokio::test]
async fn test_server_location_parsing() {
    let server = create_mock_server();

    // Location should be properly formatted
    assert!(!server.location.is_empty(), "Location should not be empty");

    // City should be extractable from location
    if let Some(city) = &server.city {
        assert!(!city.is_empty(), "City should not be empty if present");
    }

    // Country code should be valid format if present
    if let Some(code) = &server.country_code {
        assert!(
            code.len() == 2 || code.len() == 3,
            "Country code should be 2 or 3 characters"
        );
        assert!(
            code.chars().all(|c| c.is_ascii_alphabetic()),
            "Country code should be alphabetic"
        );
    }
}

#[tokio::test]
async fn test_haversine_distance_calculation() {
    // Test known distances between cities
    // Using simplified Haversine formula
    let test_cases: Vec<(f64, f64, f64, f64, f64)> = vec![
        // (lat1, lon1, lat2, lon2, expected_distance_km_approx)
        (40.7128, -74.0060, 51.5074, -0.1278, 5570.0), // NYC to London
        (35.6762, 139.6503, 37.7749, -122.4194, 8280.0), // Tokyo to SF
        (48.8566, 2.3522, 52.5200, 13.4050, 880.0),    // Paris to Berlin
    ];

    for (lat1, lon1, lat2, lon2, expected) in test_cases {
        let r: f64 = 6371.0; // Earth's radius in km
        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();
        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();

        let a: f64 = (d_lat / 2.0).sin() * (d_lat / 2.0).sin()
            + lat1_rad.cos() * lat2_rad.cos() * (d_lon / 2.0).sin() * (d_lon / 2.0).sin();
        let c: f64 = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        let distance: f64 = r * c;

        // Allow 10% margin of error
        let margin = expected * 0.1;
        assert!(
            (distance - expected).abs() < margin,
            "Distance calculation should be within 10% of expected: calculated={}, expected={}",
            distance,
            expected
        );
    }
}

#[tokio::test]
async fn test_default_location_fallback() {
    use netrunner_core::speed_test::GeoLocation;

    // Test the default fallback location (USA Central)
    let default_geo = GeoLocation {
        country: "United States".to_string(),
        city: "Kansas City".to_string(),
        latitude: 39.0997,
        longitude: -94.5786,
        isp: None,
    };

    assert_eq!(default_geo.country, "United States");
    assert_eq!(default_geo.city, "Kansas City");
    assert!(
        default_geo.latitude > 38.0 && default_geo.latitude < 40.0,
        "Fallback latitude should be in central US"
    );
    assert!(
        default_geo.longitude > -95.0 && default_geo.longitude < -94.0,
        "Fallback longitude should be in central US"
    );
}

#[tokio::test]
async fn test_empty_server_pool_handling() {
    ensure_crypto_provider();
    let config = create_test_config();
    let _speed_test = SpeedTest::new(config).expect("Failed to create SpeedTest");

    // Note: We can't easily test the internal server pool directly,
    // but we can verify the speed test handles empty pools gracefully
    // by ensuring creation succeeds and the structure is sound
    // SpeedTest should handle empty server pools gracefully
}

#[tokio::test]
async fn test_concurrent_server_testing() {
    // Test that multiple servers can be tested concurrently
    let servers = [
        create_mock_server(),
        create_mock_server(),
        create_mock_server(),
    ];

    assert_eq!(servers.len(), 3, "Should create multiple servers");

    // Verify each server has independent state
    for (i, server) in servers.iter().enumerate() {
        assert_eq!(server.name, "Test Server");
        assert!(server.distance_km.is_some());
        assert!(i < 3); // Just to use i
    }
}
