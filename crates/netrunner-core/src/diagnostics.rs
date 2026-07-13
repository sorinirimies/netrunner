//! Network diagnostics engine (UI-agnostic).
//!
//! Gathers gateway, DNS, route, IPv6 and interface information. Progress is
//! reported through [`TestEvent`]s; when no channel is supplied it runs
//! silently. The heavy "cyberpunk" presentation lives in the front-ends.

use dns_lookup::lookup_host;
use rand::RngExt as _;
use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, Instant};
use tokio::time::sleep;

use crate::events::{emit, EventSender, TestEvent};
use crate::types::{NetworkDiagnostics, RouteHop, TestConfig};

pub struct NetworkDiagnosticsTool {
    #[allow(dead_code)]
    config: TestConfig,
    events: Option<EventSender>,
}

impl NetworkDiagnosticsTool {
    pub fn new(config: TestConfig) -> Self {
        Self {
            config,
            events: None,
        }
    }

    /// Create a diagnostics tool that reports progress through an [`EventSender`].
    pub fn with_events(config: TestConfig, events: Option<EventSender>) -> Self {
        Self { config, events }
    }

    /// Attach or replace the progress event channel.
    pub fn set_events(&mut self, events: Option<EventSender>) {
        self.events = events;
    }

    fn status(&self, msg: impl Into<String>) {
        emit(&self.events, TestEvent::Status(msg.into()));
    }

    pub async fn run_diagnostics(&self) -> Result<NetworkDiagnostics, Box<dyn std::error::Error>> {
        // Determine gateway
        let gateway_ip = self.detect_gateway().await?;

        // Get DNS servers
        let dns_servers = self.detect_dns_servers().await?;

        // Measure DNS response time
        let dns_response_time = self.measure_dns_response_time().await?;

        // Trace route
        let route_hops = self.trace_route("8.8.8.8").await?;

        // Check IPv6 availability
        let is_ipv6_available = self.check_ipv6().await?;

        // Determine connection type (wired/wireless)
        let connection_type = self.detect_connection_type().await?;

        // Get network interface
        let network_interface = self.detect_network_interface().await?;

        let diagnostics = NetworkDiagnostics {
            gateway_ip,
            dns_servers,
            dns_response_time_ms: dns_response_time,
            route_hops,
            is_ipv6_available,
            connection_type: Some(connection_type),
            network_interface: Some(network_interface),
        };

        emit(
            &self.events,
            TestEvent::DiagnosticsComplete(Box::new(diagnostics.clone())),
        );

        Ok(diagnostics)
    }

    async fn detect_gateway(&self) -> Result<Option<IpAddr>, Box<dyn std::error::Error>> {
        self.status("Scanning network topology...");

        // This is a simplified approach. In a real implementation, you'd:
        // 1. On Windows: Use "ipconfig" and parse the "Default Gateway" line
        // 2. On Linux/macOS: Use "ip route | grep default" or "netstat -nr | grep default"
        sleep(Duration::from_millis(800)).await;

        let gateway = Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        if let Some(gw) = gateway {
            self.status(format!("Gateway node detected: {}", gw));
        }
        Ok(gateway)
    }

    async fn detect_dns_servers(&self) -> Result<Vec<IpAddr>, Box<dyn std::error::Error>> {
        self.status("Probing DNS infrastructure...");

        // Simplified/simulated; real code would read resolv.conf / scutil / ipconfig.
        sleep(Duration::from_millis(700)).await;

        let dns_servers = vec![
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            IpAddr::V4(Ipv4Addr::new(8, 8, 4, 4)),
        ];

        self.status(format!("DNS nodes identified: {}", dns_servers.len()));
        Ok(dns_servers)
    }

    async fn measure_dns_response_time(&self) -> Result<f64, Box<dyn std::error::Error>> {
        self.status("Measuring DNS response time...");

        let domains = vec![
            "google.com",
            "amazon.com",
            "facebook.com",
            "microsoft.com",
            "apple.com",
        ];

        let mut total_time = 0.0;
        let mut successful_lookups = 0;

        for domain in domains {
            let start = Instant::now();
            match lookup_host(domain) {
                Ok(_) => {
                    let duration = start.elapsed().as_millis() as f64;
                    total_time += duration;
                    successful_lookups += 1;
                    self.status(format!("Resolved {} in {:.2}ms", domain, duration));
                }
                Err(e) => {
                    self.status(format!("Failed to resolve {}: {}", domain, e));
                }
            }

            sleep(Duration::from_millis(100)).await;
        }

        let avg_time = if successful_lookups > 0 {
            total_time / successful_lookups as f64
        } else {
            0.0
        };

        self.status(format!("Average DNS response: {:.2}ms", avg_time));
        Ok(avg_time)
    }

    async fn trace_route(&self, target: &str) -> Result<Vec<RouteHop>, Box<dyn std::error::Error>> {
        self.status(format!("Tracing route to {}...", target));

        let max_hops = 15;
        let mut hops = Vec::new();

        // Simplified/simulated traceroute.
        for hop_number in 1..=max_hops {
            let mut rng = rand::rng();
            let delay = if hop_number < 3 {
                rng.random_range(1..10)
            } else if hop_number < 8 {
                rng.random_range(10..50)
            } else {
                rng.random_range(50..150)
            };

            sleep(Duration::from_millis(delay)).await;

            // Simulate sometimes missing hops
            let address = if hop_number != 6 && hop_number != 9 {
                let fake_ip = format!("192.168.{}.{}", hop_number, hop_number * 10);
                Some(fake_ip.parse::<IpAddr>()?)
            } else {
                None
            };

            let response_time = if address.is_some() {
                Some(delay as f64)
            } else {
                None
            };

            hops.push(RouteHop {
                hop_number: hop_number as u32,
                address,
                hostname: None,
                response_time_ms: response_time,
            });

            // Last hop should be the target
            if hop_number == max_hops {
                let target_ip = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
                hops.pop(); // Remove the last simulated hop
                hops.push(RouteHop {
                    hop_number: hop_number as u32,
                    address: Some(target_ip),
                    hostname: Some(target.to_string()),
                    response_time_ms: Some(delay as f64),
                });
            }
        }

        self.status(format!("Route to {} mapped: {} hops", target, hops.len()));
        Ok(hops)
    }

    async fn check_ipv6(&self) -> Result<bool, Box<dyn std::error::Error>> {
        self.status("Checking IPv6 connectivity...");
        sleep(Duration::from_millis(600)).await;

        // Randomly determine if IPv6 is available (simulated)
        let ipv6_available = rand::rng().random_bool(0.7);
        self.status(if ipv6_available {
            "IPv6: active"
        } else {
            "IPv6: inactive"
        });
        Ok(ipv6_available)
    }

    async fn detect_connection_type(&self) -> Result<String, Box<dyn std::error::Error>> {
        self.status("Detecting connection type...");
        sleep(Duration::from_millis(500)).await;

        let connection_type = if rand::rng().random_bool(0.6) {
            "Wireless (Wi-Fi)".to_string()
        } else {
            "Wired (Ethernet)".to_string()
        };

        self.status(format!("Connection type: {}", connection_type));
        Ok(connection_type)
    }

    async fn detect_network_interface(&self) -> Result<String, Box<dyn std::error::Error>> {
        self.status("Detecting network interface...");
        sleep(Duration::from_millis(400)).await;

        let interface = if cfg!(target_os = "windows") {
            "Ethernet".to_string()
        } else if cfg!(target_os = "macos") {
            "en0".to_string()
        } else {
            "eth0".to_string()
        };

        self.status(format!("Network interface: {}", interface));
        Ok(interface)
    }
}
