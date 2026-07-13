use chrono::Utc;
use netrunner_core::types::*;
use std::net::{IpAddr, Ipv4Addr};

#[test]
fn test_connection_quality_from_speed_and_ping() {
    // Test Excellent quality
    let quality = ConnectionQuality::from_speed_and_ping(150.0, 25.0, 15.0);
    assert_eq!(quality, ConnectionQuality::Excellent);

    // Test Good quality
    let quality = ConnectionQuality::from_speed_and_ping(60.0, 12.0, 40.0);
    assert_eq!(quality, ConnectionQuality::Good);

    // Test Average quality
    let quality = ConnectionQuality::from_speed_and_ping(30.0, 7.0, 90.0);
    assert_eq!(quality, ConnectionQuality::Average);

    // Test Poor quality
    let quality = ConnectionQuality::from_speed_and_ping(12.0, 3.0, 140.0);
    assert_eq!(quality, ConnectionQuality::Poor);

    // Test Very Poor quality
    let quality = ConnectionQuality::from_speed_and_ping(8.0, 1.5, 200.0);
    assert_eq!(quality, ConnectionQuality::VeryPoor);

    // Test Failed quality
    let quality = ConnectionQuality::from_speed_and_ping(0.0, 0.0, 0.0);
    assert_eq!(quality, ConnectionQuality::Failed);
}

#[test]
fn test_connection_quality_edge_cases() {
    // Test exact boundary conditions
    let quality = ConnectionQuality::from_speed_and_ping(100.0, 20.0, 19.99);
    assert_eq!(quality, ConnectionQuality::Excellent);

    // Just below excellent threshold
    let quality = ConnectionQuality::from_speed_and_ping(99.99, 19.99, 20.01);
    assert_eq!(quality, ConnectionQuality::Good);

    // Test with very high speeds but poor latency
    let quality = ConnectionQuality::from_speed_and_ping(1000.0, 100.0, 500.0);
    assert_eq!(quality, ConnectionQuality::VeryPoor);

    // Test with good speeds but no upload
    let quality = ConnectionQuality::from_speed_and_ping(100.0, 0.0, 20.0);
    assert_eq!(quality, ConnectionQuality::Failed);

    // Test asymmetric connection (good download, poor upload)
    let quality = ConnectionQuality::from_speed_and_ping(100.0, 1.0, 30.0);
    assert_eq!(quality, ConnectionQuality::VeryPoor);
}

#[test]
fn test_speed_test_result_default() {
    let result = SpeedTestResult::default();

    assert_eq!(result.download_mbps, 0.0);
    assert_eq!(result.upload_mbps, 0.0);
    assert_eq!(result.ping_ms, 0.0);
    assert_eq!(result.jitter_ms, 0.0);
    assert_eq!(result.packet_loss_percent, 0.0);
    assert_eq!(result.server_location, "Unknown");
    assert_eq!(result.server_ip, None);
    assert_eq!(result.client_ip, None);
    assert_eq!(result.quality, ConnectionQuality::Failed);
    assert_eq!(result.test_duration_seconds, 0.0);
    assert_eq!(result.isp, None);

    // Check that timestamp is recent (within last few seconds)
    let now = Utc::now();
    let diff = now
        .signed_duration_since(result.timestamp)
        .num_seconds()
        .abs();
    assert!(diff <= 5, "Timestamp should be recent");
}

#[test]
fn test_speed_test_result_with_values() {
    let test_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let timestamp = Utc::now();

    let result = SpeedTestResult {
        timestamp,
        download_mbps: 75.5,
        upload_mbps: 15.2,
        ping_ms: 25.8,
        jitter_ms: 3.1,
        packet_loss_percent: 0.5,
        server_location: "Test Server".to_string(),
        server_ip: Some(test_ip),
        client_ip: Some(test_ip),
        quality: ConnectionQuality::Good,
        test_duration_seconds: 12.5,
        isp: Some("Test ISP".to_string()),
    };

    assert_eq!(result.download_mbps, 75.5);
    assert_eq!(result.upload_mbps, 15.2);
    assert_eq!(result.ping_ms, 25.8);
    assert_eq!(result.jitter_ms, 3.1);
    assert_eq!(result.packet_loss_percent, 0.5);
    assert_eq!(result.server_location, "Test Server");
    assert_eq!(result.server_ip, Some(test_ip));
    assert_eq!(result.client_ip, Some(test_ip));
    assert_eq!(result.quality, ConnectionQuality::Good);
    assert_eq!(result.test_duration_seconds, 12.5);
    assert_eq!(result.isp, Some("Test ISP".to_string()));
    assert_eq!(result.timestamp, timestamp);
}

#[test]
fn test_test_server_creation() {
    let server = TestServer {
        name: "Test Server".to_string(),
        url: "https://test.example.com".to_string(),
        location: "Test Location".to_string(),
        distance_km: Some(150.5),
        latency_ms: Some(25.0),
        provider: ServerProvider::Custom("Test".to_string()),
        capabilities: ServerCapabilities {
            supports_download: true,
            supports_upload: true,
            supports_latency: true,
            max_test_size_mb: 100,
            geographic_weight: 0.5,
        },
        quality_score: Some(0.8),
        country_code: Some("US".to_string()),
        city: Some("Test City".to_string()),
        is_backup: false,
    };

    assert_eq!(server.name, "Test Server");
    assert_eq!(server.url, "https://test.example.com");
    assert_eq!(server.location, "Test Location");
    assert_eq!(server.distance_km, Some(150.5));
    assert_eq!(server.latency_ms, Some(25.0));
}

#[test]
fn test_test_config_default() {
    let config = TestConfig::default();

    assert_eq!(config.server_url, "https://httpbin.org");
    assert_eq!(config.test_size_mb, 10);
    assert_eq!(config.timeout_seconds, 30);
    assert!(!config.json_output);
    assert!(config.animation_enabled);
    assert_eq!(config.detail_level, DetailLevel::Standard);
    assert_eq!(config.max_servers, 3);
}

#[test]
fn test_test_config_custom() {
    let config = TestConfig {
        server_url: "https://custom.server.com".to_string(),
        test_size_mb: 50,
        timeout_seconds: 60,
        json_output: true,
        animation_enabled: false,
        detail_level: DetailLevel::Detailed,
        max_servers: 5,
    };

    assert_eq!(config.server_url, "https://custom.server.com");
    assert_eq!(config.test_size_mb, 50);
    assert_eq!(config.timeout_seconds, 60);
    assert!(config.json_output);
    assert!(!config.animation_enabled);
    assert_eq!(config.detail_level, DetailLevel::Detailed);
    assert_eq!(config.max_servers, 5);
}

#[test]
fn test_detail_level_ordering() {
    // Test that DetailLevel implements PartialOrd correctly
    assert!(DetailLevel::Basic < DetailLevel::Standard);
    assert!(DetailLevel::Standard < DetailLevel::Detailed);
    assert!(DetailLevel::Detailed < DetailLevel::Debug);

    assert!(DetailLevel::Debug > DetailLevel::Detailed);
    assert!(DetailLevel::Detailed > DetailLevel::Standard);
    assert!(DetailLevel::Standard > DetailLevel::Basic);
}

#[test]
fn test_connection_quality_display() {
    assert_eq!(format!("{}", ConnectionQuality::Excellent), "Excellent");
    assert_eq!(format!("{}", ConnectionQuality::Good), "Good");
    assert_eq!(format!("{}", ConnectionQuality::Average), "Average");
    assert_eq!(format!("{}", ConnectionQuality::Poor), "Poor");
    assert_eq!(format!("{}", ConnectionQuality::VeryPoor), "Very Poor");
    assert_eq!(format!("{}", ConnectionQuality::Failed), "Failed");
}

#[test]
fn test_detail_level_display() {
    assert_eq!(format!("{}", DetailLevel::Basic), "Basic");
    assert_eq!(format!("{}", DetailLevel::Standard), "Standard");
    assert_eq!(format!("{}", DetailLevel::Detailed), "Detailed");
    assert_eq!(format!("{}", DetailLevel::Debug), "Debug");
}

#[test]
fn test_route_hop_creation() {
    let hop = RouteHop {
        hop_number: 5,
        address: Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))),
        hostname: Some("gateway.example.com".to_string()),
        response_time_ms: Some(15.5),
    };

    assert_eq!(hop.hop_number, 5);
    assert_eq!(hop.address, Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
    assert_eq!(hop.hostname, Some("gateway.example.com".to_string()));
    assert_eq!(hop.response_time_ms, Some(15.5));
}

#[test]
fn test_network_diagnostics_creation() {
    let diagnostics = NetworkDiagnostics {
        gateway_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))),
        dns_servers: vec![
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            IpAddr::V4(Ipv4Addr::new(8, 8, 4, 4)),
        ],
        dns_response_time_ms: 25.0,
        route_hops: vec![],
        is_ipv6_available: true,
        connection_type: Some("Ethernet".to_string()),
        network_interface: Some("eth0".to_string()),
    };

    assert_eq!(
        diagnostics.gateway_ip,
        Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)))
    );
    assert_eq!(diagnostics.dns_servers.len(), 2);
    assert_eq!(diagnostics.dns_response_time_ms, 25.0);
    assert!(diagnostics.is_ipv6_available);
    assert_eq!(diagnostics.connection_type, Some("Ethernet".to_string()));
    assert_eq!(diagnostics.network_interface, Some("eth0".to_string()));
}

#[test]
fn test_enhanced_speed_test_result_with_jitter_and_packet_loss() {
    let result = SpeedTestResult {
        timestamp: Utc::now(),
        download_mbps: 85.7,
        upload_mbps: 18.3,
        ping_ms: 22.1,
        jitter_ms: 4.2,
        packet_loss_percent: 0.8,
        server_location: "Enhanced Test Server".to_string(),
        server_ip: Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1))),
        client_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 150))),
        quality: ConnectionQuality::Good,
        test_duration_seconds: 25.4,
        isp: Some("Enhanced ISP Provider".to_string()),
    };

    // Verify enhanced metrics are properly stored
    assert_eq!(result.jitter_ms, 4.2);
    assert_eq!(result.packet_loss_percent, 0.8);

    // Verify quality assessment
    let expected_quality = ConnectionQuality::from_speed_and_ping(
        result.download_mbps,
        result.upload_mbps,
        result.ping_ms,
    );
    assert_eq!(result.quality, expected_quality);

    // Verify network addresses
    assert!(result.server_ip.is_some());
    assert!(result.client_ip.is_some());
}

#[test]
fn test_server_provider_comprehensive_matching() {
    // Test all server provider variants
    assert_eq!(ServerProvider::Cloudflare, ServerProvider::Cloudflare);
    assert_eq!(ServerProvider::Google, ServerProvider::Google);
    assert_eq!(ServerProvider::Netflix, ServerProvider::Netflix);
    assert_eq!(ServerProvider::Ookla, ServerProvider::Ookla);

    let custom1 = ServerProvider::Custom("OVH".to_string());
    let custom2 = ServerProvider::Custom("OVH".to_string());
    let custom3 = ServerProvider::Custom("DigitalOcean".to_string());

    assert_eq!(custom1, custom2);
    assert_ne!(custom1, custom3);
    assert_ne!(ServerProvider::Cloudflare, custom1);
}

#[test]
fn test_test_server_with_quality_metrics() {
    let server = TestServer {
        name: "High Quality Server".to_string(),
        url: "https://speed.example.com".to_string(),
        location: "San Francisco, CA".to_string(),
        distance_km: Some(245.8),
        latency_ms: Some(18.5),
        provider: ServerProvider::Custom("ExampleNet".to_string()),
        capabilities: ServerCapabilities {
            supports_download: true,
            supports_upload: true,
            supports_latency: true,
            max_test_size_mb: 2000,
            geographic_weight: 0.92,
        },
        quality_score: Some(0.88),
        country_code: Some("US".to_string()),
        city: Some("San Francisco".to_string()),
        is_backup: false,
    };

    // Test all fields are properly set
    assert_eq!(server.distance_km, Some(245.8));
    assert_eq!(server.latency_ms, Some(18.5));
    assert_eq!(server.quality_score, Some(0.88));
    assert!(server.capabilities.supports_download);
    assert!(server.capabilities.supports_upload);
    assert!(server.capabilities.supports_latency);
    assert_eq!(server.capabilities.max_test_size_mb, 2000);
    assert!(server.capabilities.geographic_weight >= 0.9);
    assert!(!server.is_backup);
}

#[test]
fn test_comprehensive_serialization() {
    let test_server = TestServer {
        name: "Serialization Test Server".to_string(),
        url: "https://test.serialize.com".to_string(),
        location: "Test Location".to_string(),
        distance_km: Some(123.45),
        latency_ms: Some(67.89),
        provider: ServerProvider::Custom("TestProvider".to_string()),
        capabilities: ServerCapabilities {
            supports_download: true,
            supports_upload: false,
            supports_latency: true,
            max_test_size_mb: 100,
            geographic_weight: 0.75,
        },
        quality_score: Some(0.82),
        country_code: Some("TEST".to_string()),
        city: Some("Test City".to_string()),
        is_backup: true,
    };

    // Test JSON serialization and deserialization
    let json = serde_json::to_string(&test_server).expect("Should serialize TestServer");
    let deserialized: TestServer =
        serde_json::from_str(&json).expect("Should deserialize TestServer");

    assert_eq!(test_server.name, deserialized.name);
    assert_eq!(test_server.url, deserialized.url);
    assert_eq!(test_server.distance_km, deserialized.distance_km);
    assert_eq!(test_server.latency_ms, deserialized.latency_ms);
    assert_eq!(test_server.provider, deserialized.provider);
    assert_eq!(test_server.quality_score, deserialized.quality_score);
    assert_eq!(test_server.is_backup, deserialized.is_backup);
}

#[test]
fn test_advanced_quality_assessment() {
    // Test edge cases for quality assessment

    // Ultra-fast connection
    let quality = ConnectionQuality::from_speed_and_ping(1000.0, 500.0, 5.0);
    assert_eq!(quality, ConnectionQuality::Excellent);

    // Fiber-like connection
    let quality = ConnectionQuality::from_speed_and_ping(500.0, 100.0, 8.0);
    assert_eq!(quality, ConnectionQuality::Excellent);

    // Asymmetric cable connection
    let quality = ConnectionQuality::from_speed_and_ping(300.0, 15.0, 35.0);
    assert_eq!(quality, ConnectionQuality::Good);

    // Satellite connection characteristics
    let quality = ConnectionQuality::from_speed_and_ping(25.0, 3.0, 600.0);
    assert_eq!(quality, ConnectionQuality::VeryPoor);

    // Mobile/LTE connection
    let quality = ConnectionQuality::from_speed_and_ping(40.0, 8.0, 80.0);
    assert_eq!(quality, ConnectionQuality::Average);
}
