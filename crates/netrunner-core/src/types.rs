use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use strum::EnumString;
use strum_macros::Display;

/// Represents the quality rating of a network connection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
pub enum ConnectionQuality {
    #[strum(to_string = "Excellent")]
    Excellent,
    #[strum(to_string = "Good")]
    Good,
    #[strum(to_string = "Average")]
    Average,
    #[strum(to_string = "Poor")]
    Poor,
    #[strum(to_string = "Very Poor")]
    VeryPoor,
    #[strum(to_string = "Failed")]
    Failed,
}

impl ConnectionQuality {
    pub fn from_speed_and_ping(download_mbps: f64, upload_mbps: f64, ping_ms: f64) -> Self {
        // Simplified rating logic
        if download_mbps >= 100.0 && upload_mbps >= 20.0 && ping_ms < 20.0 {
            ConnectionQuality::Excellent
        } else if download_mbps >= 50.0 && upload_mbps >= 10.0 && ping_ms < 50.0 {
            ConnectionQuality::Good
        } else if download_mbps >= 25.0 && upload_mbps >= 5.0 && ping_ms < 100.0 {
            ConnectionQuality::Average
        } else if download_mbps >= 10.0 && upload_mbps >= 2.0 && ping_ms < 150.0 {
            ConnectionQuality::Poor
        } else if download_mbps > 0.0 && upload_mbps > 0.0 {
            ConnectionQuality::VeryPoor
        } else {
            ConnectionQuality::Failed
        }
    }
}

/// Represents a single network speed test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestResult {
    pub timestamp: DateTime<Utc>,
    pub download_mbps: f64,
    pub upload_mbps: f64,
    pub ping_ms: f64,
    pub jitter_ms: f64,
    pub packet_loss_percent: f64,
    pub server_location: String,
    pub server_ip: Option<IpAddr>,
    pub client_ip: Option<IpAddr>,
    pub quality: ConnectionQuality,
    pub test_duration_seconds: f64,
    pub isp: Option<String>,
}

impl Default for SpeedTestResult {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            download_mbps: 0.0,
            upload_mbps: 0.0,
            ping_ms: 0.0,
            jitter_ms: 0.0,
            packet_loss_percent: 0.0,
            server_location: "Unknown".to_string(),
            server_ip: None,
            client_ip: None,
            quality: ConnectionQuality::Failed,
            test_duration_seconds: 0.0,
            isp: None,
        }
    }
}

/// Represents a test server for speed testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestServer {
    pub name: String,
    pub url: String,
    pub location: String,
    pub distance_km: Option<f64>,
    pub latency_ms: Option<f64>,
    pub provider: ServerProvider,
    pub capabilities: ServerCapabilities,
    pub quality_score: Option<f64>,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub is_backup: bool,
}

/// Different server providers for speed testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerProvider {
    Cloudflare,
    Google,
    Netflix,
    Ookla,
    Custom(String),
}

/// Server capabilities for different types of tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub supports_download: bool,
    pub supports_upload: bool,
    pub supports_latency: bool,
    pub max_test_size_mb: u64,
    pub geographic_weight: f64, // Higher means better for geographic tests
}

/// Represents detailed network diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDiagnostics {
    pub gateway_ip: Option<IpAddr>,
    pub dns_servers: Vec<IpAddr>,
    pub dns_response_time_ms: f64,
    pub route_hops: Vec<RouteHop>,
    pub is_ipv6_available: bool,
    pub connection_type: Option<String>,
    pub network_interface: Option<String>,
}

/// Represents a single hop in a network route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteHop {
    pub hop_number: u32,
    pub address: Option<IpAddr>,
    pub hostname: Option<String>,
    pub response_time_ms: Option<f64>,
}

/// Configuration for the speed test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub server_url: String,
    pub test_size_mb: u64,
    pub timeout_seconds: u64,
    pub json_output: bool,
    pub animation_enabled: bool,
    pub detail_level: DetailLevel,
    pub max_servers: usize,
}

/// Level of detail for test output
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Display, EnumString,
)]
pub enum DetailLevel {
    #[strum(to_string = "Basic")]
    Basic,
    #[strum(to_string = "Standard")]
    Standard,
    #[strum(to_string = "Detailed")]
    Detailed,
    #[strum(to_string = "Debug")]
    Debug,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            server_url: "https://httpbin.org".to_string(),
            test_size_mb: 10,
            timeout_seconds: 30,
            json_output: false,
            animation_enabled: true,
            detail_level: DetailLevel::Standard,
            max_servers: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_connection_quality_boundary_conditions() {
        // Test boundary for Excellent
        let quality = ConnectionQuality::from_speed_and_ping(100.0, 20.0, 19.9);
        assert_eq!(quality, ConnectionQuality::Excellent);

        // Test boundary for Good
        let quality = ConnectionQuality::from_speed_and_ping(50.0, 10.0, 49.9);
        assert_eq!(quality, ConnectionQuality::Good);

        // Test boundary for Average
        let quality = ConnectionQuality::from_speed_and_ping(25.0, 5.0, 99.9);
        assert_eq!(quality, ConnectionQuality::Average);
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
    fn test_detail_level_ordering() {
        assert!(DetailLevel::Basic < DetailLevel::Standard);
        assert!(DetailLevel::Standard < DetailLevel::Detailed);
        assert!(DetailLevel::Detailed < DetailLevel::Debug);
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
}
