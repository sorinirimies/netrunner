use chrono::Utc;
use netrunner_core::types::{ConnectionQuality, SpeedTestResult};
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
fn test_speed_test_result_creation() {
    let test_result = SpeedTestResult {
        timestamp: Utc::now(),
        download_mbps: 75.5,
        upload_mbps: 15.2,
        ping_ms: 25.8,
        jitter_ms: 3.1,
        packet_loss_percent: 0.5,
        server_location: "Test Server".to_string(),
        server_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))),
        client_ip: Some(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))),
        quality: ConnectionQuality::Good,
        test_duration_seconds: 12.5,
        isp: Some("Test ISP".to_string()),
    };

    assert_eq!(test_result.download_mbps, 75.5);
    assert_eq!(test_result.upload_mbps, 15.2);
    assert_eq!(test_result.ping_ms, 25.8);
    assert_eq!(test_result.server_location, "Test Server");
    assert_eq!(test_result.quality, ConnectionQuality::Good);
    assert_eq!(test_result.isp, Some("Test ISP".to_string()));
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
}

#[test]
fn test_connection_quality_boundary_conditions() {
    // Test boundary for Excellent
    let quality = ConnectionQuality::from_speed_and_ping(100.0, 20.0, 19.0);
    assert_eq!(quality, ConnectionQuality::Excellent);

    // Test boundary for Good
    let quality = ConnectionQuality::from_speed_and_ping(50.0, 10.0, 49.0);
    assert_eq!(quality, ConnectionQuality::Good);

    // Test boundary for Average
    let quality = ConnectionQuality::from_speed_and_ping(25.0, 5.0, 99.0);
    assert_eq!(quality, ConnectionQuality::Average);
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
