//! History Storage - Using Redb Embedded Database
//!
//! A robust, fast, and efficient history storage system using the redb embedded database.
//! Features:
//! - ACID transactions
//! - Type-safe table definitions
//! - Crash recovery
//! - Compact storage

use chrono::{DateTime, Utc};
use redb::{ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::types::SpeedTestResult;

const DB_NAME: &str = "netrunner_history.db";
const RETENTION_DAYS: i64 = 30;

const RESULTS_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("test_results");
const STATS_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("statistics");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestStatistics {
    pub test_count: usize,
    pub avg_download_mbps: f64,
    pub max_download_mbps: f64,
    pub min_download_mbps: f64,
    pub avg_upload_mbps: f64,
    pub max_upload_mbps: f64,
    pub min_upload_mbps: f64,
    pub avg_ping_ms: f64,
    pub min_ping_ms: f64,
    pub max_ping_ms: f64,
    pub total_data_downloaded_gb: f64,
    pub total_data_uploaded_gb: f64,
    pub first_test: DateTime<Utc>,
    pub last_test: DateTime<Utc>,
}

impl Default for TestStatistics {
    fn default() -> Self {
        Self {
            test_count: 0,
            avg_download_mbps: 0.0,
            max_download_mbps: 0.0,
            min_download_mbps: f64::MAX,
            avg_upload_mbps: 0.0,
            max_upload_mbps: 0.0,
            min_upload_mbps: f64::MAX,
            avg_ping_ms: 0.0,
            min_ping_ms: f64::MAX,
            max_ping_ms: 0.0,
            total_data_downloaded_gb: 0.0,
            total_data_uploaded_gb: 0.0,
            first_test: Utc::now(),
            last_test: Utc::now(),
        }
    }
}

pub struct HistoryStorage {
    db: redb::Database,
}

#[allow(dead_code)]
impl HistoryStorage {
    /// Create a new history storage instance
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = Self::get_db_path()?;
        let db = redb::Database::create(db_path)?;

        Ok(Self { db })
    }

    /// Create a new history storage instance with custom path (for testing)
    #[cfg(test)]
    fn new_with_path(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let db = redb::Database::create(path)?;
        Ok(Self { db })
    }

    /// Get the database path
    fn get_db_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Failed to find config directory")?
            .join("netrunner");

        std::fs::create_dir_all(&config_dir)?;
        let db_path = config_dir.join(DB_NAME);

        // Migration guard: old (pre-redb) versions stored history in a *sled*
        // database, which is a DIRECTORY at this same path. redb needs a file,
        // so `Database::create` would fail with "Is a directory". The sled data
        // is unreadable by current code (the dependency was removed), so remove
        // the stale directory to unblock redb. A real redb database is a file,
        // never a directory, so this never deletes valid history.
        if db_path.is_dir() {
            let _ = std::fs::remove_dir_all(&db_path);
        }

        Ok(db_path)
    }

    /// Save a test result
    pub fn save_result(&self, result: &SpeedTestResult) -> Result<(), Box<dyn std::error::Error>> {
        // Use timestamp as key (nanoseconds since epoch for uniqueness)
        let key = result
            .timestamp
            .timestamp_nanos_opt()
            .unwrap_or_default()
            .to_be_bytes();

        // Serialize result
        let value = postcard::to_stdvec(result)?;

        // Store in database
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(RESULTS_TABLE)?;
            table.insert(key.as_slice(), value.as_slice())?;
        }
        txn.commit()?;

        // Update statistics
        self.update_statistics(result)?;

        // Clean up old records (older than 30 days)
        self.cleanup_old_records()?;

        Ok(())
    }

    /// Get recent test results
    pub fn get_recent_results(
        &self,
        limit: usize,
    ) -> Result<Vec<SpeedTestResult>, Box<dyn std::error::Error>> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(RESULTS_TABLE)?;

        let mut results = Vec::new();

        // Iterate in reverse (newest first) — skip any records that cannot be
        // decoded (e.g. stale bytes written by an older version).
        for item in table.iter()?.rev() {
            if results.len() >= limit {
                break;
            }
            let (_, value) = item?;
            if let Ok(result) = postcard::from_bytes::<SpeedTestResult>(value.value()) {
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Get all test results
    pub fn get_all_results(&self) -> Result<Vec<SpeedTestResult>, Box<dyn std::error::Error>> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(RESULTS_TABLE)?;

        let mut results = Vec::new();

        for item in table.iter()?.rev() {
            let (_, value) = item?;
            if let Ok(result) = postcard::from_bytes::<SpeedTestResult>(value.value()) {
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Get results within a date range
    pub fn get_results_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<SpeedTestResult>, Box<dyn std::error::Error>> {
        let txn = self.db.begin_read()?;
        let table = txn.open_table(RESULTS_TABLE)?;

        let start_key = start
            .timestamp_nanos_opt()
            .unwrap_or_default()
            .to_be_bytes();
        let end_key = end.timestamp_nanos_opt().unwrap_or_default().to_be_bytes();

        let mut results = Vec::new();

        let start_slice: &[u8] = start_key.as_slice();
        let end_slice: &[u8] = end_key.as_slice();

        for item in table.range(start_slice..=end_slice)? {
            let (_, value) = item?;
            if let Ok(result) = postcard::from_bytes::<SpeedTestResult>(value.value()) {
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Get results filtered by quality
    pub fn get_results_by_quality(
        &self,
        quality: crate::types::ConnectionQuality,
    ) -> Result<Vec<SpeedTestResult>, Box<dyn std::error::Error>> {
        let all_results = self.get_all_results()?;

        Ok(all_results
            .into_iter()
            .filter(|r| r.quality == quality)
            .collect())
    }

    /// Get results by server location
    pub fn get_results_by_server(
        &self,
        server_location: &str,
    ) -> Result<Vec<SpeedTestResult>, Box<dyn std::error::Error>> {
        let all_results = self.get_all_results()?;

        Ok(all_results
            .into_iter()
            .filter(|r| r.server_location.contains(server_location))
            .collect())
    }

    /// Update statistics
    fn update_statistics(
        &self,
        result: &SpeedTestResult,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut stats = self.get_statistics_internal()?;

        // Update counts
        stats.test_count += 1;

        // Update download stats
        stats.avg_download_mbps = (stats.avg_download_mbps * (stats.test_count - 1) as f64
            + result.download_mbps)
            / stats.test_count as f64;
        stats.max_download_mbps = stats.max_download_mbps.max(result.download_mbps);
        stats.min_download_mbps = stats.min_download_mbps.min(result.download_mbps);

        // Update upload stats
        stats.avg_upload_mbps = (stats.avg_upload_mbps * (stats.test_count - 1) as f64
            + result.upload_mbps)
            / stats.test_count as f64;
        stats.max_upload_mbps = stats.max_upload_mbps.max(result.upload_mbps);
        stats.min_upload_mbps = stats.min_upload_mbps.min(result.upload_mbps);

        // Update ping stats
        stats.avg_ping_ms = (stats.avg_ping_ms * (stats.test_count - 1) as f64 + result.ping_ms)
            / stats.test_count as f64;
        stats.min_ping_ms = stats.min_ping_ms.min(result.ping_ms);
        stats.max_ping_ms = stats.max_ping_ms.max(result.ping_ms);

        // Estimate data transferred (rough calculation based on test duration and speed)
        let test_duration_hours = result.test_duration_seconds / 3600.0;
        stats.total_data_downloaded_gb += result.download_mbps * test_duration_hours / 8.0 / 1000.0;
        stats.total_data_uploaded_gb += result.upload_mbps * test_duration_hours / 8.0 / 1000.0;

        // Update timestamps
        stats.last_test = result.timestamp;
        if stats.test_count == 1 {
            stats.first_test = result.timestamp;
        }

        // Save updated statistics
        let value = postcard::to_stdvec(&stats)?;
        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(STATS_TABLE)?;
            table.insert(b"global".as_slice(), value.as_slice())?;
        }
        txn.commit()?;

        Ok(())
    }

    /// Get statistics
    pub fn get_statistics(&self) -> Result<TestStatistics, Box<dyn std::error::Error>> {
        self.get_statistics_internal()
    }

    fn get_statistics_internal(&self) -> Result<TestStatistics, Box<dyn std::error::Error>> {
        let txn = self.db.begin_read()?;
        let table = match txn.open_table(STATS_TABLE) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(TestStatistics::default()),
            Err(e) => return Err(e.into()),
        };

        match table.get(b"global".as_slice())? {
            // Fall back to default if the stored bytes cannot be decoded
            // (e.g. stale bytes written by an older version).
            Some(value) => Ok(postcard::from_bytes(value.value()).unwrap_or_default()),
            None => Ok(TestStatistics::default()),
        }
    }

    /// Get statistics for a specific date range
    pub fn get_statistics_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<TestStatistics, Box<dyn std::error::Error>> {
        let results = self.get_results_by_date_range(start, end)?;

        if results.is_empty() {
            return Ok(TestStatistics::default());
        }

        let mut stats = TestStatistics {
            test_count: results.len(),
            max_download_mbps: 0.0,
            min_download_mbps: f64::MAX,
            max_upload_mbps: 0.0,
            min_upload_mbps: f64::MAX,
            ..Default::default()
        };

        // Calculate statistics
        let mut total_download = 0.0;
        let mut total_upload = 0.0;
        let mut total_ping = 0.0;
        stats.max_ping_ms = 0.0;
        stats.min_ping_ms = f64::MAX;

        for result in &results {
            total_download += result.download_mbps;
            total_upload += result.upload_mbps;
            total_ping += result.ping_ms;

            stats.max_download_mbps = stats.max_download_mbps.max(result.download_mbps);
            stats.min_download_mbps = stats.min_download_mbps.min(result.download_mbps);
            stats.max_upload_mbps = stats.max_upload_mbps.max(result.upload_mbps);
            stats.min_upload_mbps = stats.min_upload_mbps.min(result.upload_mbps);
            stats.max_ping_ms = stats.max_ping_ms.max(result.ping_ms);
            stats.min_ping_ms = stats.min_ping_ms.min(result.ping_ms);

            // Estimate data transferred
            let test_duration_hours = result.test_duration_seconds / 3600.0;
            stats.total_data_downloaded_gb +=
                result.download_mbps * test_duration_hours / 8.0 / 1000.0;
            stats.total_data_uploaded_gb += result.upload_mbps * test_duration_hours / 8.0 / 1000.0;
        }

        stats.avg_download_mbps = total_download / results.len() as f64;
        stats.avg_upload_mbps = total_upload / results.len() as f64;
        stats.avg_ping_ms = total_ping / results.len() as f64;

        if let Some(first) = results.last() {
            stats.first_test = first.timestamp;
        }
        if let Some(last) = results.first() {
            stats.last_test = last.timestamp;
        }

        Ok(stats)
    }

    /// Get the number of stored results
    pub fn count(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let txn = self.db.begin_read()?;
        let table = match txn.open_table(RESULTS_TABLE) {
            Ok(t) => t,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(0),
            Err(e) => return Err(e.into()),
        };
        Ok(table.len()? as usize)
    }

    /// Delete a specific result
    pub fn delete_result(
        &self,
        timestamp: DateTime<Utc>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let key = timestamp
            .timestamp_nanos_opt()
            .unwrap_or_default()
            .to_be_bytes();

        let txn = self.db.begin_write()?;
        {
            let mut table = txn.open_table(RESULTS_TABLE)?;
            table.remove(key.as_slice())?;
        }
        txn.commit()?;

        // Recalculate statistics
        self.recalculate_statistics()?;

        Ok(())
    }

    /// Clear all history
    pub fn clear_history(&self) -> Result<(), Box<dyn std::error::Error>> {
        let txn = self.db.begin_write()?;
        txn.delete_table(RESULTS_TABLE)?;
        txn.delete_table(STATS_TABLE)?;
        txn.commit()?;

        Ok(())
    }

    /// Recalculate all statistics from scratch
    fn recalculate_statistics(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Clear stats table
        let txn = self.db.begin_write()?;
        txn.delete_table(STATS_TABLE)?;
        txn.commit()?;

        let results = self.get_all_results()?;

        for result in results {
            self.update_statistics(&result)?;
        }

        Ok(())
    }

    /// Clean up records older than the retention period (30 days)
    fn cleanup_old_records(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Calculate cutoff timestamp (30 days ago)
        let cutoff = Utc::now() - chrono::Duration::days(RETENTION_DAYS);
        let cutoff_nanos = cutoff.timestamp_nanos_opt().unwrap_or_default();

        // Collect keys to delete
        let mut keys_to_delete = Vec::new();

        {
            let txn = self.db.begin_read()?;
            let table = match txn.open_table(RESULTS_TABLE) {
                Ok(t) => t,
                Err(redb::TableError::TableDoesNotExist(_)) => return Ok(()),
                Err(e) => return Err(e.into()),
            };

            for item in table.iter()? {
                let (_, value) = item?;

                // Deserialize to check timestamp
                if let Ok(result) = postcard::from_bytes::<SpeedTestResult>(value.value()) {
                    let result_nanos = result.timestamp.timestamp_nanos_opt().unwrap_or_default();

                    if result_nanos < cutoff_nanos {
                        keys_to_delete.push(
                            result
                                .timestamp
                                .timestamp_nanos_opt()
                                .unwrap_or_default()
                                .to_be_bytes(),
                        );
                    }
                }
            }
        }

        let deleted_count = keys_to_delete.len();

        // Delete old records
        if deleted_count > 0 {
            let txn = self.db.begin_write()?;
            {
                let mut table = txn.open_table(RESULTS_TABLE)?;
                for key in &keys_to_delete {
                    table.remove(key.as_slice())?;
                }
            }
            txn.commit()?;

            // Recalculate statistics
            self.recalculate_statistics()?;
        }

        Ok(())
    }

    /// Export history to JSON
    pub fn export_to_json(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let results = self.get_all_results()?;
        let json = serde_json::to_string_pretty(&results)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Import history from JSON
    pub fn import_from_json(&self, path: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let results: Vec<SpeedTestResult> = serde_json::from_str(&json)?;

        let count = results.len();

        for result in results {
            self.save_result(&result)?;
        }

        Ok(count)
    }

    /// Get database statistics
    pub fn get_db_stats(&self) -> Result<DbStats, Box<dyn std::error::Error>> {
        let db_path = Self::get_db_path()?;
        let size_on_disk = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
        let results_count = self.count()?;

        Ok(DbStats {
            size_on_disk,
            results_count,
            db_path: db_path.to_string_lossy().to_string(),
        })
    }

    /// Optimize database
    /// Note: redb's compact() requires &mut self which is not available through &self.
    /// The database already manages its storage efficiently with ACID transactions.
    pub fn optimize(&self) -> Result<(), Box<dyn std::error::Error>> {
        // redb handles storage management internally; no explicit optimization needed.
        Ok(())
    }

    /// Get fastest recorded download speed
    pub fn get_fastest_download(
        &self,
    ) -> Result<Option<SpeedTestResult>, Box<dyn std::error::Error>> {
        let results = self.get_all_results()?;
        Ok(results.into_iter().max_by(|a, b| {
            a.download_mbps
                .partial_cmp(&b.download_mbps)
                .unwrap_or(std::cmp::Ordering::Equal)
        }))
    }

    /// Get fastest recorded upload speed
    pub fn get_fastest_upload(
        &self,
    ) -> Result<Option<SpeedTestResult>, Box<dyn std::error::Error>> {
        let results = self.get_all_results()?;
        Ok(results.into_iter().max_by(|a, b| {
            a.upload_mbps
                .partial_cmp(&b.upload_mbps)
                .unwrap_or(std::cmp::Ordering::Equal)
        }))
    }

    /// Get lowest recorded ping
    pub fn get_lowest_ping(&self) -> Result<Option<SpeedTestResult>, Box<dyn std::error::Error>> {
        let results = self.get_all_results()?;
        Ok(results.into_iter().min_by(|a, b| {
            a.ping_ms
                .partial_cmp(&b.ping_ms)
                .unwrap_or(std::cmp::Ordering::Equal)
        }))
    }

    /// Manually cleanup old records (older than retention period)
    /// Returns the number of records deleted
    pub fn cleanup_old_records_manual(&self) -> Result<usize, Box<dyn std::error::Error>> {
        // Calculate cutoff timestamp (30 days ago)
        let cutoff = Utc::now() - chrono::Duration::days(RETENTION_DAYS);
        let cutoff_nanos = cutoff.timestamp_nanos_opt().unwrap_or_default();

        // Collect keys to delete
        let mut keys_to_delete = Vec::new();

        {
            let txn = self.db.begin_read()?;
            let table = match txn.open_table(RESULTS_TABLE) {
                Ok(t) => t,
                Err(redb::TableError::TableDoesNotExist(_)) => return Ok(0),
                Err(e) => return Err(e.into()),
            };

            for item in table.iter()? {
                let (_, value) = item?;

                // Deserialize to check timestamp
                if let Ok(result) = postcard::from_bytes::<SpeedTestResult>(value.value()) {
                    let result_nanos = result.timestamp.timestamp_nanos_opt().unwrap_or_default();

                    if result_nanos < cutoff_nanos {
                        keys_to_delete.push(
                            result
                                .timestamp
                                .timestamp_nanos_opt()
                                .unwrap_or_default()
                                .to_be_bytes(),
                        );
                    }
                }
            }
        }

        let deleted_count = keys_to_delete.len();

        // Delete old records
        if deleted_count > 0 {
            let txn = self.db.begin_write()?;
            {
                let mut table = txn.open_table(RESULTS_TABLE)?;
                for key in &keys_to_delete {
                    table.remove(key.as_slice())?;
                }
            }
            txn.commit()?;

            // Recalculate statistics
            self.recalculate_statistics()?;
        }

        Ok(deleted_count)
    }

    /// Get the retention period in days
    pub const fn get_retention_days() -> i64 {
        RETENTION_DAYS
    }

    /// Get speed trends (compares recent results to historical average)
    pub fn get_speed_trends(&self) -> Result<SpeedTrends, Box<dyn std::error::Error>> {
        let all_stats = self.get_statistics()?;
        let recent_results = self.get_recent_results(10)?;

        if recent_results.is_empty() {
            return Ok(SpeedTrends::default());
        }

        let recent_avg_download = recent_results.iter().map(|r| r.download_mbps).sum::<f64>()
            / recent_results.len() as f64;
        let recent_avg_upload =
            recent_results.iter().map(|r| r.upload_mbps).sum::<f64>() / recent_results.len() as f64;
        let recent_avg_ping =
            recent_results.iter().map(|r| r.ping_ms).sum::<f64>() / recent_results.len() as f64;

        let download_trend = if all_stats.avg_download_mbps > 0.0 {
            ((recent_avg_download - all_stats.avg_download_mbps) / all_stats.avg_download_mbps)
                * 100.0
        } else {
            0.0
        };

        let upload_trend = if all_stats.avg_upload_mbps > 0.0 {
            ((recent_avg_upload - all_stats.avg_upload_mbps) / all_stats.avg_upload_mbps) * 100.0
        } else {
            0.0
        };

        let ping_trend = if all_stats.avg_ping_ms > 0.0 {
            ((recent_avg_ping - all_stats.avg_ping_ms) / all_stats.avg_ping_ms) * 100.0
        } else {
            0.0
        };

        Ok(SpeedTrends {
            download_trend_percent: download_trend,
            upload_trend_percent: upload_trend,
            ping_trend_percent: ping_trend,
            improving: download_trend > 0.0 && upload_trend > 0.0 && ping_trend < 0.0,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct DbStats {
    pub size_on_disk: u64,
    pub results_count: usize,
    pub db_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct SpeedTrends {
    pub download_trend_percent: f64,
    pub upload_trend_percent: f64,
    pub ping_trend_percent: f64,
    pub improving: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ConnectionQuality;
    use tempfile::tempdir;

    #[test]
    fn migrates_over_stale_sled_directory() {
        // Old (pre-redb) installs left a sled DB *directory* at the redb path.
        // Opening must remove it and create a working redb file.
        let tmp = std::env::temp_dir().join(format!("nr_sled_migrate_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("HOME", &tmp);
        std::env::set_var("XDG_CONFIG_HOME", tmp.join(".config"));

        // Fabricate a stale sled-style directory at the DB path.
        let cfg = dirs::config_dir().unwrap().join("netrunner");
        std::fs::create_dir_all(&cfg).unwrap();
        let stale = cfg.join(DB_NAME);
        std::fs::create_dir_all(stale.join("blobs")).unwrap();
        std::fs::write(stale.join("conf"), b"stale").unwrap();
        assert!(stale.is_dir(), "precondition: stale sled dir exists");

        let store = HistoryStorage::new().expect("should open after migrating");
        let r = SpeedTestResult {
            download_mbps: 42.0,
            ..Default::default()
        };
        store.save_result(&r).unwrap();
        assert_eq!(store.get_recent_results(10).unwrap().len(), 1);
        assert!(
            HistoryStorage::get_db_path().unwrap().is_file(),
            "redb path should now be a file"
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_storage_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let storage = HistoryStorage::new_with_path(db_path);
        assert!(storage.is_ok());
    }

    #[test]
    fn test_save_and_retrieve() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let storage = HistoryStorage::new_with_path(db_path).unwrap();

        let result = SpeedTestResult {
            timestamp: Utc::now(),
            download_mbps: 100.0,
            upload_mbps: 50.0,
            ping_ms: 10.0,
            jitter_ms: 1.0,
            packet_loss_percent: 0.0,
            server_location: "Test Server".to_string(),
            server_ip: None,
            client_ip: None,
            quality: ConnectionQuality::Excellent,
            test_duration_seconds: 10.0,
            isp: None,
        };

        assert!(storage.save_result(&result).is_ok());

        let results = storage.get_recent_results(1).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].download_mbps, 100.0);
    }

    #[test]
    fn test_statistics() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let storage = HistoryStorage::new_with_path(db_path).unwrap();

        let stats = storage.get_statistics();
        assert!(stats.is_ok());
    }
}
