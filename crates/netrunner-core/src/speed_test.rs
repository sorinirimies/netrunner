//! Speed Test Module
//!
//! A robust, high-performance speed-test implementation tuned for accuracy
//! (results comparable to speedtest.net):
//! - Throughput is measured against a reliable anycast backend
//!   (`speed.cloudflare.com`, which routes to the nearest edge and actually
//!   implements the download/upload protocol) rather than a mix of discovered
//!   servers that may not serve the endpoint.
//! - 50 parallel connections to saturate fast links.
//! - Lock-free (atomic) byte counting, so the measurement itself is not the
//!   bottleneck at gigabit speeds.
//! - Excludes the TCP slow-start warmup window and reports a trimmed,
//!   steady-state *sustained* throughput.
//! - Geolocation-based server selection is still used for latency/ping.
//! - Support for speeds up to 10 Gbps, with graceful failure (0 Mbps ->
//!   `ConnectionQuality::Failed`) instead of a misleading floor.

use chrono::Utc;
use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::events::{emit, EventSender, Phase, SelectedServer, TestEvent};
use crate::types::{
    ConnectionQuality, ServerCapabilities, ServerProvider, SpeedTestResult, TestConfig, TestServer,
};

const PARALLEL_CONNECTIONS: usize = 50;
const SERVER_SELECTION_COUNT: usize = 3;
/// Total wall-clock duration of a throughput phase.
const TEST_SECS: u64 = 15;
/// Initial window excluded from the final throughput calc (TCP slow-start).
const WARMUP_SECS: f64 = 3.0;
/// Reliable, anycast throughput backend: Cloudflare's speed-test endpoints,
/// which route to the nearest edge and actually implement `/__down` + `/__up`.
/// Measuring against one well-provisioned backend — rather than a mix of
/// discovered servers that may not implement the download protocol — is what
/// makes the numbers comparable to speedtest.net. (The previous code appended
/// `/__down` to every server, but only Cloudflare serves it, so half the
/// connections transferred nothing and the result was badly under-reported.)
const DOWNLOAD_ENDPOINT: &str = "https://speed.cloudflare.com/__down?bytes=100000000";
const UPLOAD_ENDPOINT: &str = "https://speed.cloudflare.com/__up";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub country: String,
    pub city: String,
    pub latitude: f64,
    pub longitude: f64,
    pub isp: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ServerPerformance {
    pub server: TestServer,
    pub latency_ms: f64,
    pub jitter_ms: f64,
    pub packet_loss: f64,
    pub download_score: f64,
    pub upload_score: f64,
    pub overall_score: f64,
}

pub struct SpeedTest {
    config: TestConfig,
    client: Client,
    events: Option<EventSender>,
    geo_location: Arc<RwLock<Option<GeoLocation>>>,
    server_pool: Arc<RwLock<Vec<TestServer>>>,
}

impl SpeedTest {
    pub fn new(config: TestConfig) -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_events(config, None)
    }

    /// Create a speed test that reports progress through an [`EventSender`].
    ///
    /// Front-ends (CLI, GUI) pass a channel here to receive live
    /// [`TestEvent`]s; passing `None` runs silently.
    pub fn with_events(
        config: TestConfig,
        events: Option<EventSender>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(100)
            .pool_idle_timeout(Duration::from_secs(120))
            .tcp_keepalive(Duration::from_secs(10))
            .http2_keep_alive_interval(Duration::from_secs(10))
            .http2_adaptive_window(true)
            .http2_initial_stream_window_size(1024 * 1024) // 1MB
            .http2_initial_connection_window_size(2 * 1024 * 1024) // 2MB
            .danger_accept_invalid_certs(false)
            .build()?;

        Ok(Self {
            config,
            client,
            events,
            geo_location: Arc::new(RwLock::new(None)),
            server_pool: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Attach or replace the progress event channel.
    pub fn set_events(&mut self, events: Option<EventSender>) {
        self.events = events;
    }

    /// Run the complete speed test with intelligent server selection
    pub async fn run_full_test(&self) -> Result<SpeedTestResult, Box<dyn std::error::Error>> {
        let start = Instant::now();

        // Phase 1: Detect location
        emit(&self.events, TestEvent::PhaseStarted(Phase::Locating));
        let geo = self.detect_location().await?;
        *self.geo_location.write().await = Some(geo.clone());

        // Phase 2: Build server pool
        emit(
            &self.events,
            TestEvent::PhaseStarted(Phase::BuildingServers),
        );
        self.build_server_pool(&geo).await?;

        // Phase 3: Select best servers
        emit(
            &self.events,
            TestEvent::PhaseStarted(Phase::SelectingServers),
        );
        let best_servers = self.select_best_servers().await?;

        emit(
            &self.events,
            TestEvent::PrimarySelected {
                name: best_servers[0].name.clone(),
                location: best_servers[0].location.clone(),
                distance_km: best_servers[0].distance_km.unwrap_or(0.0),
            },
        );

        // Phase 4: Measure latency
        let ping_ms = self.measure_latency(&best_servers[0]).await?;

        // Phase 5: Download test (progressive)
        let download_mbps = self.progressive_download_test().await?;

        // Phase 6: Upload test (progressive)
        let upload_mbps = self.progressive_upload_test().await?;

        // Phase 7: Calculate statistics
        emit(&self.events, TestEvent::PhaseStarted(Phase::Jitter));
        let (jitter_ms, packet_loss) = self.measure_jitter_and_loss(&best_servers[0]).await?;
        emit(
            &self.events,
            TestEvent::JitterComplete {
                jitter_ms,
                packet_loss_percent: packet_loss,
            },
        );

        let quality = ConnectionQuality::from_speed_and_ping(download_mbps, upload_mbps, ping_ms);
        let test_duration = start.elapsed().as_secs_f64();

        let result = SpeedTestResult {
            timestamp: Utc::now(),
            download_mbps,
            upload_mbps,
            ping_ms,
            jitter_ms,
            packet_loss_percent: packet_loss,
            server_location: best_servers[0].location.clone(),
            server_ip: self.resolve_server_ip(&best_servers[0].url).await,
            client_ip: self.get_client_ip().await,
            quality,
            test_duration_seconds: test_duration,
            isp: geo.isp.clone(),
        };

        emit(&self.events, TestEvent::Completed(Box::new(result.clone())));

        Ok(result)
    }

    /// Detect user's geolocation using multiple services
    async fn detect_location(&self) -> Result<GeoLocation, Box<dyn std::error::Error>> {
        emit(
            &self.events,
            TestEvent::Status("Detecting your location...".to_string()),
        );

        // Try multiple geolocation services sequentially (first success wins)
        // Try ipapi.co
        match self.try_ipapi_co().await {
            Ok(geo) => {
                self.emit_location(&geo, "ipapi.co");
                return Ok(geo);
            }
            Err(e) => {
                // Log error at trace level for debugging
                if std::env::var("NETRUNNER_DEBUG").is_ok() {
                    eprintln!("[TRACE] ipapi.co geolocation failed: {}", e);
                }
            }
        }

        // Try ip-api.com
        match self.try_ip_api_com().await {
            Ok(geo) => {
                self.emit_location(&geo, "ip-api.com");
                return Ok(geo);
            }
            Err(e) => {
                // Log error at trace level for debugging
                if std::env::var("NETRUNNER_DEBUG").is_ok() {
                    eprintln!("[TRACE] ip-api.com geolocation failed: {}", e);
                }
            }
        }

        // Try ipinfo.io
        match self.try_ipinfo_io().await {
            Ok(geo) => {
                self.emit_location(&geo, "ipinfo.io");
                return Ok(geo);
            }
            Err(e) => {
                // Log error at trace level for debugging
                if std::env::var("NETRUNNER_DEBUG").is_ok() {
                    eprintln!("[TRACE] ipinfo.io geolocation failed: {}", e);
                }
            }
        }

        // Try freegeoip.app
        match self.try_freegeoip_app().await {
            Ok(geo) => {
                self.emit_location(&geo, "freegeoip.app");
                return Ok(geo);
            }
            Err(e) => {
                // Log error at trace level for debugging
                if std::env::var("NETRUNNER_DEBUG").is_ok() {
                    eprintln!("[TRACE] freegeoip.app geolocation failed: {}", e);
                }
            }
        }

        // Try ipwhois.app
        match self.try_ipwhois_app().await {
            Ok(geo) => {
                self.emit_location(&geo, "ipwhois.app");
                return Ok(geo);
            }
            Err(e) => {
                // Log error at trace level for debugging
                if std::env::var("NETRUNNER_DEBUG").is_ok() {
                    eprintln!("[TRACE] ipwhois.app geolocation failed: {}", e);
                }
            }
        }

        // Fallback: Use a default location (USA central) if all services fail
        emit(
            &self.events,
            TestEvent::Status(
                "Using default location (USA Central) - all geolocation services failed"
                    .to_string(),
            ),
        );

        Ok(GeoLocation {
            country: "United States".to_string(),
            city: "Kansas City".to_string(),
            latitude: 39.0997,
            longitude: -94.5786,
            isp: None,
        })
    }

    /// Emit a [`TestEvent::LocationDetected`] for a resolved location.
    fn emit_location(&self, geo: &GeoLocation, source: &str) {
        emit(
            &self.events,
            TestEvent::LocationDetected {
                city: geo.city.clone(),
                country: geo.country.clone(),
                isp: geo.isp.clone(),
                source: source.to_string(),
            },
        );
    }

    /// Fetch and parse a geolocation provider's JSON response (5s timeout).
    async fn fetch_geo_json(
        &self,
        url: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }
        Ok(response.json().await?)
    }

    async fn try_ipapi_co(&self) -> Result<GeoLocation, Box<dyn std::error::Error>> {
        let json = self.fetch_geo_json("https://ipapi.co/json/").await?;
        if json.get("error").is_some() {
            return Err(format!(
                "API error: {}",
                json["reason"].as_str().unwrap_or("Unknown")
            )
            .into());
        }
        build_geo(
            geo_str_field(&json, "country_name")?,
            geo_str_field(&json, "city")?,
            json["latitude"].as_f64().ok_or("Invalid latitude")?,
            json["longitude"].as_f64().ok_or("Invalid longitude")?,
            json["org"].as_str().map(String::from),
        )
    }

    async fn try_ip_api_com(&self) -> Result<GeoLocation, Box<dyn std::error::Error>> {
        let json = self
            .fetch_geo_json(
                "http://ip-api.com/json/?fields=status,message,country,city,lat,lon,isp",
            )
            .await?;
        if json["status"].as_str() != Some("success") {
            return Err(format!(
                "API error: {}",
                json["message"].as_str().unwrap_or("Unknown")
            )
            .into());
        }
        build_geo(
            geo_str_field(&json, "country")?,
            geo_str_field(&json, "city")?,
            json["lat"].as_f64().ok_or("Invalid latitude")?,
            json["lon"].as_f64().ok_or("Invalid longitude")?,
            json["isp"].as_str().map(String::from),
        )
    }

    async fn try_ipinfo_io(&self) -> Result<GeoLocation, Box<dyn std::error::Error>> {
        let json = self.fetch_geo_json("https://ipinfo.io/json").await?;
        // ipinfo.io returns "lat,lon" in the "loc" field.
        let (latitude, longitude) =
            parse_latlon_pair(json["loc"].as_str().ok_or("Invalid location")?)?;
        build_geo(
            geo_str_field(&json, "country")?,
            geo_str_field(&json, "city")?,
            latitude,
            longitude,
            json["org"].as_str().map(String::from),
        )
    }

    async fn try_freegeoip_app(&self) -> Result<GeoLocation, Box<dyn std::error::Error>> {
        let json = self.fetch_geo_json("https://freegeoip.app/json/").await?;
        build_geo(
            geo_str_field(&json, "country_name")?,
            geo_str_field(&json, "city")?,
            json["latitude"].as_f64().ok_or("Invalid latitude")?,
            json["longitude"].as_f64().ok_or("Invalid longitude")?,
            None,
        )
    }

    async fn try_ipwhois_app(&self) -> Result<GeoLocation, Box<dyn std::error::Error>> {
        let json = self.fetch_geo_json("https://ipwho.is/").await?;
        if !json["success"].as_bool().unwrap_or(false) {
            return Err(format!(
                "API error: {}",
                json["message"].as_str().unwrap_or("Unknown")
            )
            .into());
        }
        build_geo(
            geo_str_field(&json, "country")?,
            geo_str_field(&json, "city")?,
            json["latitude"].as_f64().ok_or("Invalid latitude")?,
            json["longitude"].as_f64().ok_or("Invalid longitude")?,
            json["connection"]["isp"].as_str().map(String::from),
        )
    }

    /// Build a comprehensive server pool based on location
    async fn build_server_pool(&self, geo: &GeoLocation) -> Result<(), Box<dyn std::error::Error>> {
        emit(
            &self.events,
            TestEvent::Status("Building server pool...".to_string()),
        );

        let mut servers = Vec::new();

        // Try dynamic server discovery first
        servers.extend(self.discover_nearby_servers(geo).await);

        // Add global CDN endpoints as fallback
        servers.extend(self.get_global_cdn_servers());

        // Calculate distances for servers that don't have them
        for server in &mut servers {
            if server.distance_km.is_none() {
                server.distance_km = Some(self.estimate_distance(geo, server));
            }
        }

        // Sort by distance (nearest first)
        servers.sort_by(|a, b| {
            a.distance_km
                .unwrap_or(f64::MAX)
                .partial_cmp(&b.distance_km.unwrap_or(f64::MAX))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Keep only the best servers
        servers.truncate(20);

        let server_count = servers.len();
        *self.server_pool.write().await = servers;

        emit(
            &self.events,
            TestEvent::ServerPoolBuilt {
                count: server_count,
            },
        );

        Ok(())
    }

    fn get_global_cdn_servers(&self) -> Vec<TestServer> {
        // Global fallback servers - used with low priority
        vec![
            TestServer {
                name: "Cloudflare Global".to_string(),
                url: "https://speed.cloudflare.com".to_string(),
                location: "Global CDN".to_string(),
                distance_km: Some(5000.0), // Lower priority than regional servers
                latency_ms: None,
                provider: ServerProvider::Cloudflare,
                capabilities: ServerCapabilities {
                    supports_download: true,
                    supports_upload: true,
                    supports_latency: true,
                    max_test_size_mb: 2000,
                    geographic_weight: 0.5, // Medium weight for global anycast
                },
                quality_score: None,
                country_code: None,
                city: None,
                is_backup: true,
            },
            TestServer {
                name: "Google Global".to_string(),
                url: "https://www.google.com".to_string(),
                location: "Global CDN".to_string(),
                distance_km: Some(5000.0),
                latency_ms: None,
                provider: ServerProvider::Google,
                capabilities: ServerCapabilities {
                    supports_download: true,
                    supports_upload: false,
                    supports_latency: true,
                    max_test_size_mb: 100,
                    geographic_weight: 0.4,
                },
                quality_score: None,
                country_code: None,
                city: None,
                is_backup: true,
            },
        ]
    }

    /// Dynamically discover nearby speed test servers based on user location
    async fn discover_nearby_servers(&self, geo: &GeoLocation) -> Vec<TestServer> {
        let mut servers = Vec::new();

        emit(
            &self.events,
            TestEvent::Status("Discovering nearby speed test servers...".to_string()),
        );

        // Try to fetch speedtest.net server list
        if let Ok(speedtest_servers) = self.fetch_speedtest_net_servers(geo).await {
            servers.extend(speedtest_servers);
        }

        // Add continent-based CDN servers
        servers.extend(self.get_continent_servers(geo));

        // Add country-specific servers
        servers.extend(self.get_country_servers(geo));

        emit(
            &self.events,
            TestEvent::NearbyServersFound {
                count: servers.len(),
            },
        );

        servers
    }

    /// Fetch real speedtest.net server list based on location
    async fn fetch_speedtest_net_servers(
        &self,
        geo: &GeoLocation,
    ) -> Result<Vec<TestServer>, Box<dyn std::error::Error>> {
        // Speedtest.net uses a JSON API to get nearby servers
        let url = "https://www.speedtest.net/api/js/servers?engine=js&limit=10";

        if let Ok(response) = self.client.get(url).send().await {
            if let Ok(text) = response.text().await {
                // Parse the response and create TestServer objects
                if let Ok(servers) = self.parse_speedtest_servers(&text, geo) {
                    return Ok(servers);
                }
            }
        }

        // Fallback: Use Open Speed Test servers
        self.get_open_speedtest_servers(geo).await
    }

    fn parse_speedtest_servers(
        &self,
        json: &str,
        geo: &GeoLocation,
    ) -> Result<Vec<TestServer>, Box<dyn std::error::Error>> {
        // Simple JSON parsing for speedtest.net format
        // Format: [{"id":123,"host":"server.host.com","lat":40.7,"lon":-74.0,"name":"New York","country":"US","sponsor":"ISP Name"}]

        let mut servers = Vec::new();

        // Use serde_json to parse
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json) {
            if let Some(array) = parsed.as_array() {
                for server in array.iter().take(10) {
                    if let (Some(host), Some(name), Some(country), Some(lat), Some(lon)) = (
                        server.get("host").and_then(|v| v.as_str()),
                        server.get("name").and_then(|v| v.as_str()),
                        server.get("country").and_then(|v| v.as_str()),
                        server.get("lat").and_then(|v| v.as_f64()),
                        server.get("lon").and_then(|v| v.as_f64()),
                    ) {
                        let distance =
                            self.calculate_distance(geo.latitude, geo.longitude, lat, lon);

                        servers.push(TestServer {
                            name: format!("{}, {}", name, country),
                            url: format!("https://{}", host),
                            location: format!("{}, {}", name, country),
                            distance_km: Some(distance),
                            latency_ms: None,
                            provider: ServerProvider::Custom(
                                host.split('.').next().unwrap_or("speedtest").to_string(),
                            ),
                            capabilities: ServerCapabilities {
                                supports_download: true,
                                supports_upload: true,
                                supports_latency: true,
                                max_test_size_mb: 1000,
                                geographic_weight: 1.0,
                            },
                            quality_score: None,
                            country_code: Some(country.to_string()),
                            city: Some(name.to_string()),
                            is_backup: false,
                        });
                    }
                }
            }
        }

        if servers.is_empty() {
            Err("No servers parsed".into())
        } else {
            Ok(servers)
        }
    }

    async fn get_open_speedtest_servers(
        &self,
        geo: &GeoLocation,
    ) -> Result<Vec<TestServer>, Box<dyn std::error::Error>> {
        // Fallback to manually curated list of high-performance servers
        let mut servers = Vec::new();

        // Major internet exchanges and data centers
        let endpoints = vec![
            (
                "Cloudflare (Anycast)",
                "https://speed.cloudflare.com",
                0.0,
                0.0,
                "Global",
            ),
            (
                "LibreSpeed DE-IX",
                "https://frankfurt.speedtest.wtnet.de",
                50.1109,
                8.6821,
                "Frankfurt, Germany",
            ),
            (
                "LibreSpeed AMS-IX",
                "https://ams.speedtest.wtnet.de",
                52.3676,
                4.9041,
                "Amsterdam, Netherlands",
            ),
            (
                "LibreSpeed Singapore",
                "https://sg.speedtest.wtnet.de",
                1.3521,
                103.8198,
                "Singapore",
            ),
            (
                "LibreSpeed New York",
                "https://nyc.speedtest.wtnet.de",
                40.7128,
                -74.0060,
                "New York, USA",
            ),
            (
                "LibreSpeed Los Angeles",
                "https://la.speedtest.wtnet.de",
                34.0522,
                -118.2437,
                "Los Angeles, USA",
            ),
            (
                "LibreSpeed Tokyo",
                "https://tyo.speedtest.wtnet.de",
                35.6762,
                139.6503,
                "Tokyo, Japan",
            ),
            (
                "LibreSpeed London",
                "https://lon.speedtest.wtnet.de",
                51.5074,
                -0.1278,
                "London, UK",
            ),
            (
                "LibreSpeed Sydney",
                "https://syd.speedtest.wtnet.de",
                -33.8688,
                151.2093,
                "Sydney, Australia",
            ),
        ];

        for (name, url, lat, lon, location) in endpoints {
            let distance = if lat == 0.0 && lon == 0.0 {
                999999.0 // Global anycast
            } else {
                self.calculate_distance(geo.latitude, geo.longitude, lat, lon)
            };

            servers.push(TestServer {
                name: name.to_string(),
                url: url.to_string(),
                location: location.to_string(),
                distance_km: Some(distance),
                latency_ms: None,
                provider: ServerProvider::Custom("LibreSpeed".to_string()),
                capabilities: ServerCapabilities {
                    supports_download: true,
                    supports_upload: true,
                    supports_latency: true,
                    max_test_size_mb: 2000,
                    geographic_weight: 0.9,
                },
                quality_score: None,
                country_code: Some(location.split(", ").last().unwrap_or("").to_string()),
                city: Some(location.split(", ").next().unwrap_or(location).to_string()),
                is_backup: false,
            });
        }

        Ok(servers)
    }

    fn get_continent_servers(&self, geo: &GeoLocation) -> Vec<TestServer> {
        let mut servers = Vec::new();

        // Determine continent based on coordinates
        let continent = self.determine_continent(geo.latitude, geo.longitude);

        match continent.as_str() {
            "North America" => {
                servers.push(self.create_server_with_coords(
                    geo,
                    "US East Coast Hub",
                    "https://ash.speedtest.wtnet.de",
                    "Ashburn, USA",
                    Some("US".to_string()),
                    39.0438,
                    -77.4874,
                ));
                servers.push(self.create_server_with_coords(
                    geo,
                    "US West Coast Hub",
                    "https://lax.speedtest.wtnet.de",
                    "Los Angeles, USA",
                    Some("US".to_string()),
                    34.0522,
                    -118.2437,
                ));
            }
            "Europe" => {
                servers.push(self.create_server_with_coords(
                    geo,
                    "Europe Central Hub",
                    "https://frankfurt.speedtest.wtnet.de",
                    "Frankfurt, Germany",
                    Some("DE".to_string()),
                    50.1109,
                    8.6821,
                ));
                servers.push(self.create_server_with_coords(
                    geo,
                    "Europe West Hub",
                    "https://lon.speedtest.wtnet.de",
                    "London, UK",
                    Some("GB".to_string()),
                    51.5074,
                    -0.1278,
                ));
            }
            "Asia" => {
                servers.push(self.create_server_with_coords(
                    geo,
                    "Asia Pacific Hub",
                    "https://sg.speedtest.wtnet.de",
                    "Singapore",
                    Some("SG".to_string()),
                    1.3521,
                    103.8198,
                ));
                servers.push(self.create_server_with_coords(
                    geo,
                    "Asia East Hub",
                    "https://tokyo.speedtest.wtnet.de",
                    "Tokyo, Japan",
                    Some("JP".to_string()),
                    35.6762,
                    139.6503,
                ));
            }
            "South America" => {
                servers.push(self.create_server_with_coords(
                    geo,
                    "South America Hub",
                    "https://saopaulo.speedtest.wtnet.de",
                    "São Paulo, Brazil",
                    Some("BR".to_string()),
                    -23.5505,
                    -46.6333,
                ));
            }
            "Africa" => {
                servers.push(self.create_server_with_coords(
                    geo,
                    "Africa Hub",
                    "https://capetown.speedtest.wtnet.de",
                    "Cape Town, South Africa",
                    Some("ZA".to_string()),
                    -33.9249,
                    18.4241,
                ));
            }
            "Oceania" => {
                servers.push(self.create_server_with_coords(
                    geo,
                    "Oceania Hub",
                    "https://syd.speedtest.wtnet.de",
                    "Sydney, Australia",
                    Some("AU".to_string()),
                    -33.8688,
                    151.2093,
                ));
            }
            _ => {}
        }

        servers
    }

    fn determine_continent(&self, lat: f64, lon: f64) -> String {
        // Simple continent determination based on coordinates
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

    fn get_country_servers(&self, geo: &GeoLocation) -> Vec<TestServer> {
        let mut servers = Vec::new();

        // Add country-specific servers based on common countries
        match geo.country.as_str() {
            "United States" | "US" => {
                servers.push(self.create_server(
                    "US Central",
                    "https://dal.speedtest.wtnet.de",
                    "Dallas, USA",
                    Some("US".to_string()),
                ));
            }
            "United Kingdom" | "GB" | "UK" => {
                servers.push(self.create_server(
                    "UK Primary",
                    "https://lon.speedtest.wtnet.de",
                    "London, UK",
                    Some("GB".to_string()),
                ));
            }
            "Germany" | "DE" => {
                servers.push(self.create_server(
                    "DE Primary",
                    "https://frankfurt.speedtest.wtnet.de",
                    "Frankfurt, Germany",
                    Some("DE".to_string()),
                ));
            }
            "France" | "FR" => {
                servers.push(self.create_server(
                    "FR Primary",
                    "https://paris.speedtest.wtnet.de",
                    "Paris, France",
                    Some("FR".to_string()),
                ));
            }
            "Japan" | "JP" => {
                servers.push(self.create_server(
                    "JP Primary",
                    "https://tyo.speedtest.wtnet.de",
                    "Tokyo, Japan",
                    Some("JP".to_string()),
                ));
            }
            "Australia" | "AU" => {
                servers.push(self.create_server(
                    "AU Primary",
                    "https://syd.speedtest.wtnet.de",
                    "Sydney, Australia",
                    Some("AU".to_string()),
                ));
            }
            "Canada" | "CA" => {
                servers.push(self.create_server(
                    "CA Primary",
                    "https://tor.speedtest.wtnet.de",
                    "Toronto, Canada",
                    Some("CA".to_string()),
                ));
            }
            _ => {}
        }

        servers
    }

    fn calculate_distance(&self, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        // Haversine formula for distance calculation
        let r = 6371.0; // Earth's radius in km
        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();
        let lat1 = lat1.to_radians();
        let lat2 = lat2.to_radians();

        let a = (d_lat / 2.0).sin() * (d_lat / 2.0).sin()
            + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin() * (d_lon / 2.0).sin();
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        r * c
    }

    #[allow(clippy::too_many_arguments)]
    fn create_server_with_coords(
        &self,
        geo: &GeoLocation,
        name: &str,
        url: &str,
        location: &str,
        country_code: Option<String>,
        lat: f64,
        lon: f64,
    ) -> TestServer {
        let distance = self.calculate_distance(geo.latitude, geo.longitude, lat, lon);

        TestServer {
            name: name.to_string(),
            url: url.to_string(),
            location: location.to_string(),
            distance_km: Some(distance),
            latency_ms: None,
            provider: ServerProvider::Custom("LibreSpeed".to_string()),
            capabilities: ServerCapabilities {
                supports_download: true,
                supports_upload: true,
                supports_latency: true,
                max_test_size_mb: 2000,
                geographic_weight: 1.0,
            },
            quality_score: None,
            country_code,
            city: Some(location.split(", ").next().unwrap_or(location).to_string()),
            is_backup: false,
        }
    }

    fn create_server(
        &self,
        name: &str,
        url: &str,
        location: &str,
        country_code: Option<String>,
    ) -> TestServer {
        TestServer {
            name: name.to_string(),
            url: url.to_string(),
            location: location.to_string(),
            distance_km: None,
            latency_ms: None,
            provider: ServerProvider::Cloudflare,
            capabilities: ServerCapabilities {
                supports_download: true,
                supports_upload: true,
                supports_latency: true,
                max_test_size_mb: 1000,
                geographic_weight: 1.2,
            },
            quality_score: None,
            country_code,
            city: Some(location.split(',').next().unwrap_or("").trim().to_string()),
            is_backup: false,
        }
    }

    fn determine_region(&self, country: &str) -> String {
        match country {
            "United States" | "Canada" | "Mexico" => "North America".to_string(),
            "United Kingdom" | "Germany" | "France" | "Spain" | "Italy" | "Netherlands"
            | "Belgium" | "Switzerland" | "Austria" | "Poland" => "Europe".to_string(),
            "Japan" | "China" | "South Korea" | "Singapore" | "Australia" | "New Zealand"
            | "India" => "Asia Pacific".to_string(),
            "Brazil" | "Argentina" | "Chile" => "South America".to_string(),
            _ => "Other".to_string(),
        }
    }

    fn estimate_distance(&self, geo: &GeoLocation, server: &TestServer) -> f64 {
        // Simplified distance estimation based on region
        // In production, use actual server coordinates
        let region = self.determine_region(&geo.country);

        if let Some(city) = &server.city {
            if city.contains(&geo.city) {
                return 10.0; // Same city
            }
        }

        match (region.as_str(), server.location.as_str()) {
            ("North America", loc) if loc.contains("USA") || loc.contains("Canada") => 500.0,
            ("Europe", loc) if loc.contains("Europe") || loc.contains("UK") => 300.0,
            ("Asia Pacific", loc) if loc.contains("Asia") || loc.contains("Japan") => 400.0,
            _ => 5000.0, // Cross-region
        }
    }

    /// Select the best servers by testing them concurrently
    async fn select_best_servers(&self) -> Result<Vec<TestServer>, Box<dyn std::error::Error>> {
        emit(
            &self.events,
            TestEvent::Status("Testing server performance...".to_string()),
        );

        let servers = self.server_pool.read().await.clone();

        if servers.is_empty() {
            return Err("No servers in pool".into());
        }

        let mut test_results = Vec::new();

        // Test servers concurrently - test up to 15 servers
        let mut futures = FuturesUnordered::new();

        for server in servers.into_iter().take(15) {
            let client = self.client.clone();
            futures.push(async move { Self::quick_latency_test(&client, &server).await });
        }

        while let Some(result) = futures.next().await {
            if let Ok(mut server) = result {
                if let Some(latency) = server.latency_ms {
                    let distance = server.distance_km.unwrap_or(1000.0);
                    let geographic_weight = server.capabilities.geographic_weight;

                    // Calculate quality score considering latency, distance, and geographic weight
                    // Lower latency and distance = higher score
                    // Formula: base_score * geographic_weight / (latency_penalty + distance_penalty)
                    let latency_penalty = latency.max(1.0); // Avoid division by near-zero
                    let distance_penalty = (distance / 100.0).max(1.0);
                    server.quality_score =
                        Some((10000.0 * geographic_weight) / (latency_penalty + distance_penalty));

                    test_results.push(server);
                }
            }
        }

        if test_results.is_empty() {
            return Err("No servers responded to latency tests".into());
        }

        // Sort by quality score (highest first)
        test_results.sort_by(|a, b| {
            b.quality_score
                .unwrap_or(0.0)
                .partial_cmp(&a.quality_score.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let selected = test_results
            .into_iter()
            .take(SERVER_SELECTION_COUNT)
            .collect::<Vec<_>>();

        if !self.config.json_output {
            emit(
                &self.events,
                TestEvent::ServersSelected {
                    servers: selected
                        .iter()
                        .map(|server| SelectedServer {
                            name: server.name.clone(),
                            location: server.location.clone(),
                            latency_ms: server.latency_ms.unwrap_or(0.0),
                            distance_km: server.distance_km.unwrap_or(0.0),
                        })
                        .collect(),
                },
            );
        }

        Ok(selected)
    }

    async fn quick_latency_test(
        client: &Client,
        server: &TestServer,
    ) -> Result<TestServer, Box<dyn std::error::Error>> {
        let mut latencies = Vec::new();
        let mut server = server.clone();

        for _ in 0..3 {
            let start = Instant::now();
            match client
                .head(&server.url)
                .timeout(Duration::from_secs(2))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() || resp.status().is_redirection() => {
                    latencies.push(start.elapsed().as_millis() as f64);
                }
                _ => {}
            }
        }

        if !latencies.is_empty() {
            server.latency_ms = Some(latencies.iter().sum::<f64>() / latencies.len() as f64);
        }

        Ok(server)
    }

    /// Progressive download test against a reliable anycast backend.
    ///
    /// Fixes vs the old implementation: uses a real speed-test endpoint (so
    /// every connection transfers data), a lock-free counter, and a
    /// warmup-excluded, outlier-trimmed *sustained* throughput — which is what
    /// speedtest.net reports.
    async fn progressive_download_test(&self) -> Result<f64, Box<dyn std::error::Error>> {
        emit(&self.events, TestEvent::PhaseStarted(Phase::Download));

        // Lock-free shared byte counter: at gigabit speeds a Mutex locked on
        // every chunk becomes the bottleneck and under-reports throughput.
        let total_bytes = Arc::new(AtomicU64::new(0));
        let start = Instant::now();
        let test_duration = Duration::from_secs(TEST_SECS);

        let mut handles = Vec::new();
        for _ in 0..PARALLEL_CONNECTIONS {
            let url = DOWNLOAD_ENDPOINT.to_string();
            let client = self.client.clone();
            let total_bytes = Arc::clone(&total_bytes);
            let test_start = start;

            let handle = tokio::spawn(async move {
                let end_time = test_start + test_duration;
                while Instant::now() < end_time {
                    match client.get(&url).send().await {
                        Ok(response) => {
                            let mut stream = response.bytes_stream();
                            while let Some(chunk_result) = stream.next().await {
                                if Instant::now() >= end_time {
                                    break;
                                }
                                if let Ok(chunk) = chunk_result {
                                    total_bytes.fetch_add(chunk.len() as u64, Ordering::Relaxed);
                                }
                            }
                        }
                        Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
                    }
                    if Instant::now() >= end_time {
                        break;
                    }
                }
            });
            handles.push(handle);
        }

        // Monitor emits live samples and returns them for the final calc.
        let events = self.events.clone();
        let total_bytes_monitor = Arc::clone(&total_bytes);
        let monitor_handle = tokio::spawn(async move {
            collect_throughput_samples(total_bytes_monitor, start, test_duration, events, false)
                .await
        });

        for handle in handles {
            let _ = handle.await;
        }
        let samples = monitor_handle.await.unwrap_or_default();

        let mbps = sustained_mbps(&samples, WARMUP_SECS).clamp(0.0, 10_000.0);
        emit(&self.events, TestEvent::DownloadComplete { mbps });
        Ok(mbps)
    }

    /// Progressive upload test against the reliable anycast backend.
    async fn progressive_upload_test(&self) -> Result<f64, Box<dyn std::error::Error>> {
        emit(&self.events, TestEvent::PhaseStarted(Phase::Upload));

        let total_bytes = Arc::new(AtomicU64::new(0));
        let start = Instant::now();
        let test_duration = Duration::from_secs(TEST_SECS);

        // 5 MB chunks streamed repeatedly for the duration of the test.
        let chunk_size = 5 * 1024 * 1024;
        let test_data = vec![0u8; chunk_size];

        let mut handles = Vec::new();
        for _ in 0..10 {
            let url = UPLOAD_ENDPOINT.to_string();
            let client = self.client.clone();
            let total_bytes = Arc::clone(&total_bytes);
            let data = test_data.clone();
            let test_start = start;

            let handle = tokio::spawn(async move {
                let end_time = test_start + test_duration;
                while Instant::now() < end_time {
                    match client
                        .post(&url)
                        .body(data.clone())
                        .timeout(Duration::from_secs(10))
                        .send()
                        .await
                    {
                        Ok(_) => {
                            total_bytes.fetch_add(data.len() as u64, Ordering::Relaxed);
                        }
                        Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
                    }
                }
            });
            handles.push(handle);
        }

        let events = self.events.clone();
        let total_bytes_monitor = Arc::clone(&total_bytes);
        let monitor_handle = tokio::spawn(async move {
            collect_throughput_samples(total_bytes_monitor, start, test_duration, events, true)
                .await
        });

        for handle in handles {
            let _ = handle.await;
        }
        let samples = monitor_handle.await.unwrap_or_default();

        let mbps = sustained_mbps(&samples, WARMUP_SECS).clamp(0.0, 10_000.0);
        emit(&self.events, TestEvent::UploadComplete { mbps });
        Ok(mbps)
    }

    async fn measure_latency(
        &self,
        server: &TestServer,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        emit(&self.events, TestEvent::PhaseStarted(Phase::Latency));

        let mut latencies = Vec::new();

        for _i in 0..10 {
            let start = Instant::now();
            match self
                .client
                .head(&server.url)
                .timeout(Duration::from_secs(2))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() || resp.status().is_redirection() => {
                    let latency = start.elapsed().as_millis() as f64;
                    latencies.push(latency);

                    let current_avg = latencies.iter().sum::<f64>() / latencies.len() as f64;
                    emit(
                        &self.events,
                        TestEvent::LatencyProgress {
                            avg_ms: current_avg,
                        },
                    );
                }
                _ => {}
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let avg_latency = if !latencies.is_empty() {
            latencies.iter().sum::<f64>() / latencies.len() as f64
        } else {
            50.0
        };

        emit(
            &self.events,
            TestEvent::LatencyComplete {
                avg_ms: avg_latency,
            },
        );

        Ok(avg_latency)
    }

    async fn measure_jitter_and_loss(
        &self,
        server: &TestServer,
    ) -> Result<(f64, f64), Box<dyn std::error::Error>> {
        let mut latencies = Vec::new();
        let mut lost = 0;
        let total = 20;

        for _ in 0..total {
            let start = Instant::now();
            match self
                .client
                .head(&server.url)
                .timeout(Duration::from_secs(1))
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() || resp.status().is_redirection() => {
                    latencies.push(start.elapsed().as_millis() as f64);
                }
                _ => {
                    lost += 1;
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        let jitter = if latencies.len() > 1 {
            let mean = latencies.iter().sum::<f64>() / latencies.len() as f64;
            let variance =
                latencies.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / latencies.len() as f64;
            variance.sqrt()
        } else {
            0.0
        };

        let packet_loss = (lost as f64 / total as f64) * 100.0;

        Ok((jitter, packet_loss))
    }

    async fn get_client_ip(&self) -> Option<IpAddr> {
        if let Ok(response) = self
            .client
            .get("https://api.ipify.org?format=json")
            .timeout(Duration::from_secs(3))
            .send()
            .await
        {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                return json["ip"].as_str().and_then(|s| s.parse::<IpAddr>().ok());
            }
        }
        None
    }

    async fn resolve_server_ip(&self, url: &str) -> Option<IpAddr> {
        if let Ok(parsed) = url.parse::<reqwest::Url>() {
            if let Some(host) = parsed.host_str() {
                if let Ok(addrs) = tokio::net::lookup_host(format!("{}:443", host)).await {
                    return addrs.into_iter().next().map(|addr| addr.ip());
                }
            }
        }
        None
    }
}

/// Extract a required, non-empty geolocation string field (rejects "Unknown").
fn geo_str_field(
    json: &serde_json::Value,
    key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    json[key]
        .as_str()
        .filter(|s| !s.is_empty() && *s != "Unknown")
        .map(str::to_string)
        .ok_or_else(|| format!("Invalid {key}").into())
}

/// Parse a `"lat,lon"` pair (as returned by ipinfo.io's `loc` field).
fn parse_latlon_pair(loc: &str) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let coords: Vec<&str> = loc.split(',').collect();
    if coords.len() != 2 {
        return Err("Invalid coordinates format".into());
    }
    let lat = coords[0].trim().parse().map_err(|_| "Invalid latitude")?;
    let lon = coords[1].trim().parse().map_err(|_| "Invalid longitude")?;
    Ok((lat, lon))
}

/// Validate coordinates and assemble a [`GeoLocation`].
///
/// `(0, 0)` is treated as invalid (a common "unknown location" sentinel).
fn build_geo(
    country: String,
    city: String,
    latitude: f64,
    longitude: f64,
    isp: Option<String>,
) -> Result<GeoLocation, Box<dyn std::error::Error>> {
    if latitude == 0.0 && longitude == 0.0 {
        return Err("Invalid coordinates".into());
    }
    Ok(GeoLocation {
        country,
        city,
        latitude,
        longitude,
        isp,
    })
}

/// Poll a shared byte counter every 200 ms, emit live throughput samples, and
/// return every `(elapsed_secs, mbps)` sample for the final calculation.
///
/// Shared by the download and upload phases (previously duplicated).
async fn collect_throughput_samples(
    total_bytes: Arc<AtomicU64>,
    start: Instant,
    test_duration: Duration,
    events: Option<EventSender>,
    is_upload: bool,
) -> Vec<(f64, f64)> {
    let mut samples = Vec::new();
    let mut last_bytes = 0u64;
    let mut last_time = Instant::now();
    let mut peak = 0.0f64;
    let end_time = start + test_duration;

    while Instant::now() < end_time {
        tokio::time::sleep(Duration::from_millis(200)).await;

        let bytes = total_bytes.load(Ordering::Relaxed);
        let time_diff = last_time.elapsed().as_secs_f64();
        if time_diff >= 0.2 {
            let bytes_diff = bytes.saturating_sub(last_bytes);
            let mbps = (bytes_diff as f64 * 8.0) / (time_diff * 1_000_000.0);
            peak = peak.max(mbps);
            let elapsed = start.elapsed().as_secs_f64();
            samples.push((elapsed, mbps));

            let event = if is_upload {
                TestEvent::UploadSample {
                    mbps,
                    peak_mbps: peak,
                    elapsed_secs: elapsed,
                }
            } else {
                TestEvent::DownloadSample {
                    mbps,
                    peak_mbps: peak,
                    elapsed_secs: elapsed,
                }
            };
            emit(&events, event);

            last_bytes = bytes;
            last_time = Instant::now();
        }
    }
    samples
}

/// Compute sustained throughput (Mbps) from per-interval `(elapsed_s, mbps)`
/// samples the way speedtest.net does: drop the TCP slow-start warmup window,
/// then trim the slowest/fastest 10% (transient dips and spikes) and average
/// the steady-state middle. Returns `0.0` when there is no usable data (a
/// genuine failure, which maps to `ConnectionQuality::Failed`).
fn sustained_mbps(samples: &[(f64, f64)], warmup_secs: f64) -> f64 {
    let mut steady: Vec<f64> = samples
        .iter()
        .filter(|(t, mbps)| *t >= warmup_secs && *mbps > 0.0)
        .map(|(_, mbps)| *mbps)
        .collect();

    // If the whole test was shorter than the warmup, fall back to any samples.
    if steady.is_empty() {
        steady = samples
            .iter()
            .map(|(_, mbps)| *mbps)
            .filter(|mbps| *mbps > 0.0)
            .collect();
    }
    if steady.is_empty() {
        return 0.0;
    }

    steady.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let trim = steady.len() / 10;
    let slice = &steady[trim..steady.len() - trim];
    let slice = if slice.is_empty() { &steady[..] } else { slice };
    slice.iter().sum::<f64>() / slice.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sustained_mbps_excludes_warmup() {
        // Ramp during warmup (<3s) is slow; steady state is ~100.
        let samples = vec![
            (0.2, 5.0),
            (1.0, 20.0),
            (2.0, 60.0),
            (3.2, 100.0),
            (4.0, 101.0),
            (5.0, 99.0),
            (6.0, 100.0),
        ];
        let mbps = sustained_mbps(&samples, WARMUP_SECS);
        assert!(
            (mbps - 100.0).abs() < 5.0,
            "expected ~100 sustained, got {mbps}"
        );
    }

    #[test]
    fn sustained_mbps_trims_outliers() {
        // One huge spike and one deep dip in steady state should be trimmed.
        let mut samples = vec![(3.5, 0.1), (3.6, 5000.0)];
        for i in 0..20 {
            samples.push((4.0 + i as f64 * 0.1, 500.0));
        }
        let mbps = sustained_mbps(&samples, WARMUP_SECS);
        assert!((mbps - 500.0).abs() < 20.0, "expected ~500, got {mbps}");
    }

    #[test]
    fn sustained_mbps_zero_when_no_data() {
        assert_eq!(sustained_mbps(&[], WARMUP_SECS), 0.0);
        assert_eq!(sustained_mbps(&[(4.0, 0.0), (5.0, 0.0)], WARMUP_SECS), 0.0);
    }

    #[test]
    fn sustained_mbps_falls_back_for_short_tests() {
        // All samples within warmup window -> still returns a positive estimate.
        let samples = vec![(0.5, 40.0), (1.0, 60.0), (1.5, 50.0)];
        let mbps = sustained_mbps(&samples, WARMUP_SECS);
        assert!(mbps > 0.0, "short test should still estimate, got {mbps}");
    }

    #[test]
    fn geo_str_field_extracts_and_rejects() {
        let json = serde_json::json!({ "city": "Berlin", "empty": "", "unk": "Unknown" });
        assert_eq!(geo_str_field(&json, "city").unwrap(), "Berlin");
        assert!(geo_str_field(&json, "empty").is_err());
        assert!(geo_str_field(&json, "unk").is_err());
        assert!(geo_str_field(&json, "missing").is_err());
    }

    #[test]
    fn parse_latlon_pair_ok_and_err() {
        assert_eq!(parse_latlon_pair("52.52,13.40").unwrap(), (52.52, 13.40));
        assert_eq!(parse_latlon_pair(" 1.0 , 2.0 ").unwrap(), (1.0, 2.0));
        assert!(parse_latlon_pair("52.52").is_err());
        assert!(parse_latlon_pair("a,b").is_err());
    }

    #[test]
    fn build_geo_rejects_zero_coords() {
        assert!(build_geo("C".into(), "City".into(), 0.0, 0.0, None).is_err());
        let g = build_geo("C".into(), "City".into(), 1.0, 2.0, Some("ISP".into())).unwrap();
        assert_eq!(g.latitude, 1.0);
        assert_eq!(g.isp.as_deref(), Some("ISP"));
    }

    #[test]
    fn test_region_determination() {
        // Install the ring crypto provider (reqwest needs a TLS backend even for unit tests)
        let _ = rustls::crypto::ring::default_provider().install_default();
        let config = TestConfig::default();
        let speed_test = SpeedTest::new(config).unwrap();

        assert_eq!(
            speed_test.determine_region("United States"),
            "North America"
        );
        assert_eq!(speed_test.determine_region("Germany"), "Europe");
        assert_eq!(speed_test.determine_region("Japan"), "Asia Pacific");
    }
}
