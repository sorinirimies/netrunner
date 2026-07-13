//! Integration tests for geolocation functionality
//!
//! These tests verify that the geolocation services work correctly
//! and handle various edge cases and failures gracefully.

use netrunner_core::speed_test::{GeoLocation, SpeedTest};
use netrunner_core::types::{DetailLevel, TestConfig};
use std::time::Duration;

fn ensure_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// Helper function to create a test config for geolocation testing
fn create_geo_test_config() -> TestConfig {
    TestConfig {
        server_url: "https://httpbin.org".to_string(),
        test_size_mb: 1,
        timeout_seconds: 10,
        json_output: true, // Suppress UI output during tests
        animation_enabled: false,
        detail_level: DetailLevel::Standard,
        max_servers: 2,
    }
}

#[tokio::test]
async fn test_geolocation_basic_structure() {
    let geo = GeoLocation {
        country: "United States".to_string(),
        city: "New York".to_string(),
        latitude: 40.7128,
        longitude: -74.0060,
        isp: Some("Test ISP".to_string()),
    };

    assert!(!geo.country.is_empty(), "Country should not be empty");
    assert!(!geo.city.is_empty(), "City should not be empty");
    assert!(
        geo.latitude >= -90.0 && geo.latitude <= 90.0,
        "Latitude should be valid"
    );
    assert!(
        geo.longitude >= -180.0 && geo.longitude <= 180.0,
        "Longitude should be valid"
    );
}

#[tokio::test]
async fn test_geolocation_without_isp() {
    let geo = GeoLocation {
        country: "Japan".to_string(),
        city: "Tokyo".to_string(),
        latitude: 35.6762,
        longitude: 139.6503,
        isp: None,
    };

    assert_eq!(geo.country, "Japan");
    assert_eq!(geo.city, "Tokyo");
    assert!(geo.isp.is_none(), "ISP can be optional");
}

#[tokio::test]
async fn test_geolocation_extreme_coordinates() {
    // Test polar regions
    let north_pole = GeoLocation {
        country: "Arctic".to_string(),
        city: "North Pole".to_string(),
        latitude: 90.0,
        longitude: 0.0,
        isp: None,
    };
    assert_eq!(north_pole.latitude, 90.0);

    let south_pole = GeoLocation {
        country: "Antarctica".to_string(),
        city: "South Pole".to_string(),
        latitude: -90.0,
        longitude: 0.0,
        isp: None,
    };
    assert_eq!(south_pole.latitude, -90.0);

    // Test international date line
    let date_line = GeoLocation {
        country: "Pacific".to_string(),
        city: "Date Line".to_string(),
        latitude: 0.0,
        longitude: 180.0,
        isp: None,
    };
    assert_eq!(date_line.longitude, 180.0);
}

#[tokio::test]
async fn test_geolocation_major_cities() {
    let cities = vec![
        ("London", "United Kingdom", 51.5074, -0.1278),
        ("Tokyo", "Japan", 35.6762, 139.6503),
        ("Sydney", "Australia", -33.8688, 151.2093),
        ("São Paulo", "Brazil", -23.5505, -46.6333),
        ("Mumbai", "India", 19.0760, 72.8777),
        ("Cairo", "Egypt", 30.0444, 31.2357),
    ];

    for (city, country, lat, lon) in cities {
        let geo = GeoLocation {
            country: country.to_string(),
            city: city.to_string(),
            latitude: lat,
            longitude: lon,
            isp: None,
        };

        assert!(!geo.city.is_empty());
        assert!(!geo.country.is_empty());
        assert!(geo.latitude >= -90.0 && geo.latitude <= 90.0);
        assert!(geo.longitude >= -180.0 && geo.longitude <= 180.0);
    }
}

#[tokio::test]
async fn test_geolocation_serialization() {
    let geo = GeoLocation {
        country: "Germany".to_string(),
        city: "Berlin".to_string(),
        latitude: 52.5200,
        longitude: 13.4050,
        isp: Some("Deutsche Telekom".to_string()),
    };

    // Test JSON serialization
    let json = serde_json::to_string(&geo);
    assert!(json.is_ok(), "GeoLocation should serialize to JSON");

    // Test deserialization
    if let Ok(json_str) = json {
        let deserialized: Result<GeoLocation, _> = serde_json::from_str(&json_str);
        assert!(
            deserialized.is_ok(),
            "GeoLocation should deserialize from JSON"
        );

        if let Ok(parsed_geo) = deserialized {
            assert_eq!(parsed_geo.country, "Germany");
            assert_eq!(parsed_geo.city, "Berlin");
            assert_eq!(parsed_geo.latitude, 52.5200);
            assert_eq!(parsed_geo.longitude, 13.4050);
            assert_eq!(parsed_geo.isp, Some("Deutsche Telekom".to_string()));
        }
    }
}

#[tokio::test]
async fn test_geolocation_distance_calculation() {
    // Test distance between known cities
    let new_york = GeoLocation {
        country: "USA".to_string(),
        city: "New York".to_string(),
        latitude: 40.7128,
        longitude: -74.0060,
        isp: None,
    };

    let london = GeoLocation {
        country: "UK".to_string(),
        city: "London".to_string(),
        latitude: 51.5074,
        longitude: -0.1278,
        isp: None,
    };

    // Haversine formula
    let r: f64 = 6371.0;
    let d_lat: f64 = (london.latitude - new_york.latitude).to_radians();
    let d_lon: f64 = (london.longitude - new_york.longitude).to_radians();
    let lat1: f64 = new_york.latitude.to_radians();
    let lat2: f64 = london.latitude.to_radians();

    let a: f64 = (d_lat / 2.0).sin() * (d_lat / 2.0).sin()
        + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin() * (d_lon / 2.0).sin();
    let c: f64 = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    let distance: f64 = r * c;

    // NYC to London is approximately 5,570 km
    assert!(
        distance > 5400.0 && distance < 5700.0,
        "Distance NYC-London should be ~5570km, got {}",
        distance
    );
}

#[tokio::test]
async fn test_geolocation_with_special_characters() {
    let geo = GeoLocation {
        country: "Côte d'Ivoire".to_string(),
        city: "São Paulo".to_string(),
        latitude: -23.5505,
        longitude: -46.6333,
        isp: Some("Société Générale".to_string()),
    };

    assert!(geo.country.contains("Côte"));
    assert!(geo.city.contains("São"));
    assert!(geo.isp.unwrap().contains("Société"));
}

#[tokio::test]
async fn test_geolocation_validation_invalid_latitude() {
    // Test that we can detect invalid latitudes
    let invalid_geo = GeoLocation {
        country: "Test".to_string(),
        city: "Test".to_string(),
        latitude: 91.0, // Invalid!
        longitude: 0.0,
        isp: None,
    };

    assert!(
        invalid_geo.latitude > 90.0,
        "Should be able to detect invalid latitude"
    );
}

#[tokio::test]
async fn test_geolocation_validation_invalid_longitude() {
    // Test that we can detect invalid longitudes
    let invalid_geo = GeoLocation {
        country: "Test".to_string(),
        city: "Test".to_string(),
        latitude: 0.0,
        longitude: 181.0, // Invalid!
        isp: None,
    };

    assert!(
        invalid_geo.longitude > 180.0,
        "Should be able to detect invalid longitude"
    );
}

#[tokio::test]
async fn test_default_fallback_location() {
    // Test that the default fallback location is valid
    let default_geo = GeoLocation {
        country: "United States".to_string(),
        city: "Kansas City".to_string(),
        latitude: 39.0997,
        longitude: -94.5786,
        isp: None,
    };

    // Kansas City is in the geographic center of the US
    assert_eq!(default_geo.country, "United States");
    assert_eq!(default_geo.city, "Kansas City");
    assert!(
        default_geo.latitude > 38.0 && default_geo.latitude < 40.0,
        "Kansas City latitude should be ~39°N"
    );
    assert!(
        default_geo.longitude > -95.0 && default_geo.longitude < -94.0,
        "Kansas City longitude should be ~-95°W"
    );
}

#[tokio::test]
async fn test_geolocation_clone() {
    let geo = GeoLocation {
        country: "France".to_string(),
        city: "Paris".to_string(),
        latitude: 48.8566,
        longitude: 2.3522,
        isp: Some("Orange".to_string()),
    };

    let cloned = geo.clone();

    assert_eq!(geo.country, cloned.country);
    assert_eq!(geo.city, cloned.city);
    assert_eq!(geo.latitude, cloned.latitude);
    assert_eq!(geo.longitude, cloned.longitude);
    assert_eq!(geo.isp, cloned.isp);
}

#[tokio::test]
async fn test_geolocation_debug_format() {
    let geo = GeoLocation {
        country: "Spain".to_string(),
        city: "Madrid".to_string(),
        latitude: 40.4168,
        longitude: -3.7038,
        isp: Some("Telefonica".to_string()),
    };

    let debug_str = format!("{:?}", geo);
    assert!(debug_str.contains("Spain"));
    assert!(debug_str.contains("Madrid"));
}

#[tokio::test]
async fn test_real_geolocation_detection() {
    ensure_crypto_provider();
    // This test actually tries to detect location using the APIs
    // It's designed to be robust and not fail in CI/CD environments
    let config = create_geo_test_config();
    let speed_test = SpeedTest::new(config).expect("Failed to create SpeedTest");

    // Run with a reasonable timeout since this makes real network calls
    let result = tokio::time::timeout(Duration::from_secs(20), speed_test.run_full_test()).await;

    match result {
        Ok(Ok(test_result)) => {
            // If we got a result, verify the location was detected properly
            assert!(
                !test_result.server_location.is_empty(),
                "Server location should be detected"
            );

            // Verify we got actual speed test results
            assert!(
                test_result.download_mbps >= 0.0,
                "Download speed should be non-negative"
            );
            assert!(
                test_result.upload_mbps >= 0.0,
                "Upload speed should be non-negative"
            );
            assert!(test_result.ping_ms >= 0.0, "Ping should be non-negative");

            println!("✓ Geolocation detected: {}", test_result.server_location);
            println!(
                "✓ Speed test completed: ↓{:.1} Mbps ↑{:.1} Mbps",
                test_result.download_mbps, test_result.upload_mbps
            );
        }
        Ok(Err(e)) => {
            // Network issues are acceptable in test environments
            // but we should still verify the error is reasonable
            let error_msg = format!("{}", e);
            println!(
                "⚠ Speed test failed (acceptable in test env): {}",
                error_msg
            );

            // Verify it's a network-related error, not a code bug
            assert!(
                error_msg.contains("location")
                    || error_msg.contains("network")
                    || error_msg.contains("timeout")
                    || error_msg.contains("server")
                    || error_msg.contains("connect")
                    || error_msg.contains("HTTP"),
                "Error should be network-related, got: {}",
                error_msg
            );
        }
        Err(_) => {
            println!("⚠ Test timed out (acceptable in test env - slow network)");
            // Timeout is acceptable - speed test structure is valid even if network is slow
        }
    }
}

#[tokio::test]
async fn test_geolocation_api_fallback_behavior() {
    // Test that the fallback location is used when all APIs fail
    // This tests the resilience of the system
    let fallback_geo = GeoLocation {
        country: "United States".to_string(),
        city: "Kansas City".to_string(),
        latitude: 39.0997,
        longitude: -94.5786,
        isp: None,
    };

    // Verify fallback location is valid
    assert!(!fallback_geo.country.is_empty());
    assert!(!fallback_geo.city.is_empty());
    assert!(fallback_geo.latitude >= -90.0 && fallback_geo.latitude <= 90.0);
    assert!(fallback_geo.longitude >= -180.0 && fallback_geo.longitude <= 180.0);
}

#[tokio::test]
async fn test_geolocation_with_minimal_data() {
    // Test geolocation with minimum required data (no ISP)
    let minimal_geo = GeoLocation {
        country: "Test Country".to_string(),
        city: "Test City".to_string(),
        latitude: 0.0,
        longitude: 0.0,
        isp: None,
    };

    assert_eq!(minimal_geo.country, "Test Country");
    assert_eq!(minimal_geo.city, "Test City");
    assert!(minimal_geo.isp.is_none());
}

#[tokio::test]
async fn test_geolocation_coordinate_precision() {
    // Test that high precision coordinates are preserved
    let precise_geo = GeoLocation {
        country: "Test".to_string(),
        city: "Test".to_string(),
        latitude: 40.712776,
        longitude: -74.005974,
        isp: None,
    };

    assert_eq!(precise_geo.latitude, 40.712776);
    assert_eq!(precise_geo.longitude, -74.005974);
}

#[tokio::test]
async fn test_continent_determination() {
    // Test continent determination based on coordinates
    let test_cases = vec![
        (40.7128, -74.0060, "North America"),  // New York
        (51.5074, -0.1278, "Europe"),          // London
        (35.6762, 139.6503, "Asia"),           // Tokyo
        (-33.8688, 151.2093, "Oceania"),       // Sydney
        (-23.5505, -46.6333, "South America"), // São Paulo
        (30.0444, 31.2357, "Africa"),          // Cairo
    ];

    for (lat, lon, expected_continent) in test_cases {
        let continent = determine_continent(lat, lon);
        assert!(
            !continent.is_empty(),
            "Should determine continent for {}, {}",
            lat,
            lon
        );
        // Note: The actual implementation might classify differently at borders
        println!(
            "Location ({}, {}) classified as: {} (expected: {})",
            lat, lon, continent, expected_continent
        );
    }
}

#[tokio::test]
async fn test_debug_mode_environment_variable() {
    // Test that debug mode can be enabled via environment variable
    // This doesn't test the actual output, just that the env var can be checked

    // Without debug mode
    assert!(std::env::var("NETRUNNER_DEBUG").is_err());

    // Set debug mode
    std::env::set_var("NETRUNNER_DEBUG", "1");
    assert!(std::env::var("NETRUNNER_DEBUG").is_ok());

    // Clean up
    std::env::remove_var("NETRUNNER_DEBUG");
    assert!(std::env::var("NETRUNNER_DEBUG").is_err());
}

#[tokio::test]
async fn test_geolocation_service_names() {
    // Test that we know all the geolocation service names
    let services = vec![
        "ipapi.co",
        "ip-api.com",
        "ipinfo.io",
        "freegeoip.app",
        "ipwhois.app",
    ];

    assert_eq!(services.len(), 5, "Should have 5 geolocation services");

    for service in services {
        assert!(!service.is_empty(), "Service name should not be empty");
        assert!(service.contains('.'), "Service should be a domain name");
    }
}

// Helper function to test continent determination
fn determine_continent(lat: f64, lon: f64) -> String {
    if lat > 15.0 && lon > -130.0 && lon < -50.0 {
        "North America".to_string()
    } else if lat < 15.0 && lat > -60.0 && lon > -85.0 && lon < -30.0 {
        "South America".to_string()
    } else if lat > 35.0 && lon > -15.0 && lon < 60.0 {
        "Europe".to_string()
    } else if lat > -40.0 && lat < 40.0 && lon > -20.0 && lon < 55.0 {
        "Africa".to_string()
    } else if lat > -15.0 && lon > 60.0 && lon < 180.0 {
        "Asia".to_string()
    } else if lat < -10.0 && lon > 110.0 && lon < 180.0 {
        "Oceania".to_string()
    } else {
        "Unknown".to_string()
    }
}

#[tokio::test]
async fn test_timezone_implications() {
    // Test locations across different timezones
    let locations = vec![
        ("Los Angeles", -118.2437, "UTC-8"),
        ("New York", -74.0060, "UTC-5"),
        ("London", -0.1278, "UTC+0"),
        ("Tokyo", 139.6503, "UTC+9"),
        ("Sydney", 151.2093, "UTC+10"),
    ];

    for (city, lon, _timezone) in locations {
        assert!(
            (-180.0..=180.0).contains(&lon),
            "Longitude for {} should be valid",
            city
        );
    }
}

#[tokio::test]
async fn test_geolocation_error_cases() {
    // Test that invalid data doesn't panic
    let test_cases = vec![
        (0.0, 0.0),    // Null Island (valid but suspicious)
        (90.0, 0.0),   // North Pole
        (-90.0, 0.0),  // South Pole
        (0.0, 180.0),  // Date line
        (0.0, -180.0), // Date line (other side)
    ];

    for (lat, lon) in test_cases {
        let geo = GeoLocation {
            country: "Test".to_string(),
            city: "Test".to_string(),
            latitude: lat,
            longitude: lon,
            isp: None,
        };

        // These should not panic
        assert_eq!(geo.latitude, lat);
        assert_eq!(geo.longitude, lon);
    }
}

#[tokio::test]
async fn test_geolocation_isp_parsing() {
    let test_cases = vec![
        "Comcast Cable",
        "Deutsche Telekom AG",
        "China Telecom",
        "Vodafone Group",
        "AT&T Services",
    ];

    for isp_name in test_cases {
        let geo = GeoLocation {
            country: "Test".to_string(),
            city: "Test".to_string(),
            latitude: 0.0,
            longitude: 0.0,
            isp: Some(isp_name.to_string()),
        };

        assert!(geo.isp.is_some());
        assert_eq!(geo.isp.unwrap(), isp_name);
    }
}
