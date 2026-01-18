//! Cache module for storing and retrieving skill scan results.
//!
//! This module provides a caching layer to avoid repeated filesystem scanning
//! by storing scan results with metadata (mtime, size, content hash) for
//! invalidation.

use crate::core::SikilError;
use crate::utils::paths::{ensure_dir_exists, get_cache_path};
use fs_err as fs;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Current schema version for the cache database.
/// Increment this when the schema changes to trigger migration/recreation.
const SCHEMA_VERSION: u32 = 1;

/// Maximum size of content hash to store (SHA256 = 64 hex chars)
const MAX_HASH_SIZE: usize = 64;

/// Trait defining cache operations for skill scan results.
pub trait Cache {
    /// Get cached scan entry for a directory path.
    ///
    /// Returns `None` if not cached or if the entry is invalid due to
    /// mtime/size mismatch.
    fn get(&self, path: &Path) -> Result<Option<ScanEntry>, SikilError>;

    /// Put a scan entry into the cache.
    ///
    /// Replaces any existing entry for the same path.
    fn put(&self, entry: &ScanEntry) -> Result<(), SikilError>;

    /// Invalidate a cached entry by path.
    fn invalidate(&self, path: &Path) -> Result<(), SikilError>;

    /// Invalidate all cached entries for paths that no longer exist.
    ///
    /// Returns the number of entries removed.
    fn clean_stale(&self) -> Result<usize, SikilError>;

    /// Clear all entries from the cache.
    fn clear(&self) -> Result<(), SikilError>;
}

/// A cached scan entry for a skill directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanEntry {
    /// Absolute path to the directory that was scanned
    pub path: PathBuf,

    /// Last modification time of the directory (for invalidation)
    pub mtime: u64,

    /// Size of the directory in bytes (for fast invalidation)
    pub size: u64,

    /// Hash of the directory contents (for content-based invalidation)
    pub content_hash: String,

    /// Timestamp when this entry was cached
    pub cached_at: u64,

    /// Parsed skill metadata from SKILL.md
    pub skill_name: Option<String>,

    /// Whether the path contains a valid skill
    pub is_valid_skill: bool,
}

/// SQLite-based cache implementation.
pub struct SqliteCache {
    conn: Connection,
}

impl SqliteCache {
    /// Open or create the cache database at the default location.
    ///
    /// Creates the database file and parent directories if they don't exist.
    /// Initializes schema or migrates if version mismatch.
    pub fn open() -> Result<Self, SikilError> {
        let cache_path = get_cache_path();
        Self::open_at(&cache_path)
    }

    /// Open or create the cache database at a specific path.
    ///
    /// Creates the database file and parent directories if they don't exist.
    /// Initializes schema or migrates if version mismatch.
    pub fn open_at(path: &Path) -> Result<Self, SikilError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            ensure_dir_exists(parent).map_err(|e| SikilError::ConfigError {
                reason: format!("failed to create cache directory: {}", e),
            })?;
        }

        let conn = Connection::open(path).map_err(|e| SikilError::ConfigError {
            reason: format!("failed to open cache database: {}", e),
        })?;

        // Enable WAL mode for better concurrent access
        conn.query_row("PRAGMA journal_mode=WAL", [], |_row| Ok(()))
            .map_err(|e| SikilError::ConfigError {
                reason: format!("failed to enable WAL mode: {}", e),
            })?;

        let mut cache = Self { conn };

        // Check and run migrations if needed
        cache.migrate()?;

        Ok(cache)
    }

    /// Initialize the database schema.
    fn init_schema(&self) -> Result<(), SikilError> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS scan_cache (
                    path TEXT PRIMARY KEY,
                    mtime INTEGER NOT NULL,
                    size INTEGER NOT NULL,
                    content_hash TEXT NOT NULL,
                    cached_at INTEGER NOT NULL,
                    skill_name TEXT,
                    is_valid_skill INTEGER NOT NULL
                )",
                [],
            )
            .map_err(|e| SikilError::ConfigError {
                reason: format!("failed to create cache table: {}", e),
            })?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_skill_name ON scan_cache(skill_name)",
                [],
            )
            .map_err(|e| SikilError::ConfigError {
                reason: format!("failed to create index: {}", e),
            })?;

        // Set schema version
        self.conn
            .execute("PRAGMA user_version = 1", [])
            .map_err(|e| SikilError::ConfigError {
                reason: format!("failed to set schema version: {}", e),
            })?;

        Ok(())
    }

    /// Check schema version and migrate if needed.
    ///
    /// For now, we simply recreate the database if the version doesn't match.
    /// Future versions will implement proper migrations.
    fn migrate(&mut self) -> Result<(), SikilError> {
        let version: u32 = self
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(|e| SikilError::ConfigError {
                reason: format!("failed to read schema version: {}", e),
            })?;

        if version == 0 {
            self.init_schema()?;
        } else if version != SCHEMA_VERSION {
            // Schema version mismatch - recreate the database
            self.conn
                .execute("DROP TABLE IF EXISTS scan_cache", [])
                .map_err(|e| SikilError::ConfigError {
                    reason: format!("failed to drop old schema: {}", e),
                })?;
            self.init_schema()?;
        }

        Ok(())
    }

    /// Get the current mtime for a path.
    fn get_path_mtime(path: &Path) -> Result<u64, SikilError> {
        let metadata = fs::metadata(path).map_err(|_e| SikilError::DirectoryNotFound {
            path: path.to_path_buf(),
        })?;

        let modified = metadata.modified().map_err(|_| SikilError::ConfigError {
            reason: format!("unable to get mtime for {}", path.display()),
        })?;

        let duration_since_epoch =
            modified
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(|_| SikilError::ConfigError {
                    reason: format!("mtime before unix epoch for {}", path.display()),
                })?;

        Ok(duration_since_epoch.as_secs())
    }

    /// Get the current timestamp as unix seconds.
    #[cfg(test)]
    fn now() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Get a cached entry without mtime validation (for testing only).
    #[cfg(test)]
    fn get_raw(&self, path: &Path) -> Result<Option<ScanEntry>, SikilError> {
        let path_str = path.to_str().ok_or_else(|| SikilError::ConfigError {
            reason: format!("invalid path: {}", path.display()),
        })?;

        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT path, mtime, size, content_hash, cached_at, skill_name, is_valid_skill
                 FROM scan_cache WHERE path = ?",
            )
            .map_err(|e| SikilError::ConfigError {
                reason: format!("failed to prepare get_raw query: {}", e),
            })?;

        let entry = stmt
            .query_row(params![path_str], |row| {
                Ok(ScanEntry {
                    path: PathBuf::from(row.get::<_, String>(0)?),
                    mtime: row.get(1)?,
                    size: row.get(2)?,
                    content_hash: row.get(3)?,
                    cached_at: row.get(4)?,
                    skill_name: row.get(5)?,
                    is_valid_skill: row.get::<_, i64>(6)? != 0,
                })
            })
            .optional()
            .map_err(|e| SikilError::ConfigError {
                reason: format!("failed to query cache: {}", e),
            })?;

        Ok(entry)
    }
}

impl Cache for SqliteCache {
    fn get(&self, path: &Path) -> Result<Option<ScanEntry>, SikilError> {
        let path_str = path.to_str().ok_or_else(|| SikilError::ConfigError {
            reason: format!("invalid path: {}", path.display()),
        })?;

        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT path, mtime, size, content_hash, cached_at, skill_name, is_valid_skill
                 FROM scan_cache WHERE path = ?",
            )
            .map_err(|e| SikilError::ConfigError {
                reason: format!("failed to prepare get query: {}", e),
            })?;

        let entry = stmt
            .query_row(params![path_str], |row| {
                Ok(ScanEntry {
                    path: PathBuf::from(row.get::<_, String>(0)?),
                    mtime: row.get(1)?,
                    size: row.get(2)?,
                    content_hash: row.get(3)?,
                    cached_at: row.get(4)?,
                    skill_name: row.get(5)?,
                    is_valid_skill: row.get::<_, i64>(6)? != 0,
                })
            })
            .optional()
            .map_err(|e| SikilError::ConfigError {
                reason: format!("failed to query cache: {}", e),
            })?;

        // Check if entry is still valid by comparing mtime and size
        if let Some(ref entry) = entry {
            // Try to get current metadata
            if let Ok(current_mtime) = Self::get_path_mtime(path) {
                if current_mtime != entry.mtime {
                    // Path has been modified, return None to trigger rescan
                    return Ok(None);
                }
            } else {
                // Path no longer exists
                return Ok(None);
            }
        }

        Ok(entry)
    }

    fn put(&self, entry: &ScanEntry) -> Result<(), SikilError> {
        let path_str = entry.path.to_str().ok_or_else(|| SikilError::ConfigError {
            reason: format!("invalid path: {}", entry.path.display()),
        })?;

        // Validate content hash length
        if entry.content_hash.len() > MAX_HASH_SIZE {
            return Err(SikilError::ValidationError {
                reason: format!("content hash too long: {}", entry.content_hash.len()),
            });
        }

        self.conn
            .execute(
                "INSERT OR REPLACE INTO scan_cache
                 (path, mtime, size, content_hash, cached_at, skill_name, is_valid_skill)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    path_str,
                    entry.mtime,
                    entry.size,
                    entry.content_hash,
                    entry.cached_at,
                    entry.skill_name,
                    entry.is_valid_skill as i64,
                ],
            )
            .map_err(|e| SikilError::ConfigError {
                reason: format!("failed to insert cache entry: {}", e),
            })?;

        Ok(())
    }

    fn invalidate(&self, path: &Path) -> Result<(), SikilError> {
        let path_str = path.to_str().ok_or_else(|| SikilError::ConfigError {
            reason: format!("invalid path: {}", path.display()),
        })?;

        self.conn
            .execute("DELETE FROM scan_cache WHERE path = ?", params![path_str])
            .map_err(|e| SikilError::ConfigError {
                reason: format!("failed to invalidate cache entry: {}", e),
            })?;

        Ok(())
    }

    fn clean_stale(&self) -> Result<usize, SikilError> {
        // Get all cached paths
        let mut stmt = self
            .conn
            .prepare("SELECT path FROM scan_cache")
            .map_err(|e| SikilError::ConfigError {
                reason: format!("failed to prepare clean_stale query: {}", e),
            })?;

        let paths: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| SikilError::ConfigError {
                reason: format!("failed to query cache paths: {}", e),
            })?
            .collect::<Result<_, _>>()
            .map_err(|e| SikilError::ConfigError {
                reason: format!("failed to read cache path: {}", e),
            })?;

        let mut stale_count = 0;

        for path_str in paths {
            let path = PathBuf::from(&path_str);

            // Check if path still exists
            if !path.exists() {
                self.conn
                    .execute("DELETE FROM scan_cache WHERE path = ?", params![path_str])
                    .map_err(|e| SikilError::ConfigError {
                        reason: format!("failed to delete stale entry: {}", e),
                    })?;

                stale_count += 1;
            }
        }

        Ok(stale_count)
    }

    fn clear(&self) -> Result<(), SikilError> {
        self.conn
            .execute("DELETE FROM scan_cache", [])
            .map_err(|e| SikilError::ConfigError {
                reason: format!("failed to clear cache: {}", e),
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    /// Helper to create a temporary cache for testing
    fn create_temp_cache() -> Result<SqliteCache, SikilError> {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.into_temp_path();
        SqliteCache::open_at(&temp_path)
    }

    #[test]
    fn test_sqlite_cache_open_creates_schema() {
        let cache = create_temp_cache().unwrap();

        // Verify table exists
        let table_exists: i64 = cache
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='scan_cache'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(table_exists, 1);

        // Verify schema version
        let version: u32 = cache
            .conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();

        assert_eq!(version, SCHEMA_VERSION);
    }

    #[test]
    fn test_cache_put_and_get() {
        let cache = create_temp_cache().unwrap();

        let entry = ScanEntry {
            path: PathBuf::from("/test/skill"),
            mtime: 1234567890,
            size: 1024,
            content_hash: "abc123".to_string(),
            cached_at: SqliteCache::now(),
            skill_name: Some("test-skill".to_string()),
            is_valid_skill: true,
        };

        // Put entry
        cache.put(&entry).unwrap();

        // Get entry (using get_raw since test path doesn't exist)
        let retrieved = cache.get_raw(&PathBuf::from("/test/skill")).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();

        assert_eq!(retrieved.path, PathBuf::from("/test/skill"));
        assert_eq!(retrieved.mtime, 1234567890);
        assert_eq!(retrieved.size, 1024);
        assert_eq!(retrieved.content_hash, "abc123");
        assert_eq!(retrieved.skill_name, Some("test-skill".to_string()));
        assert!(retrieved.is_valid_skill);
    }

    #[test]
    fn test_cache_get_returns_none_for_nonexistent() {
        let cache = create_temp_cache().unwrap();

        // Using get_raw to test cache lookup without mtime validation
        let result = cache.get_raw(&PathBuf::from("/nonexistent/path")).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_invalidate() {
        let cache = create_temp_cache().unwrap();

        let entry = ScanEntry {
            path: PathBuf::from("/test/skill"),
            mtime: 1234567890,
            size: 1024,
            content_hash: "abc123".to_string(),
            cached_at: SqliteCache::now(),
            skill_name: Some("test-skill".to_string()),
            is_valid_skill: true,
        };

        cache.put(&entry).unwrap();
        assert!(cache
            .get_raw(&PathBuf::from("/test/skill"))
            .unwrap()
            .is_some());

        cache.invalidate(&PathBuf::from("/test/skill")).unwrap();
        assert!(cache
            .get_raw(&PathBuf::from("/test/skill"))
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_cache_clear() {
        let cache = create_temp_cache().unwrap();

        // Add multiple entries
        for i in 0..5 {
            let entry = ScanEntry {
                path: PathBuf::from(format!("/test/skill{}", i)),
                mtime: 1234567890 + i as u64,
                size: 1024,
                content_hash: format!("hash{}", i),
                cached_at: SqliteCache::now(),
                skill_name: Some(format!("skill{}", i)),
                is_valid_skill: true,
            };
            cache.put(&entry).unwrap();
        }

        // Verify entries exist (using get_raw)
        assert!(cache
            .get_raw(&PathBuf::from("/test/skill0"))
            .unwrap()
            .is_some());

        // Clear all
        cache.clear().unwrap();

        // Verify all entries are gone
        for i in 0..5 {
            let result = cache
                .get_raw(&PathBuf::from(format!("/test/skill{}", i)))
                .unwrap();
            assert!(result.is_none());
        }
    }

    #[test]
    fn test_cache_clean_stale() {
        let cache = create_temp_cache().unwrap();

        // Create a temporary directory that will exist
        let temp_dir = std::env::temp_dir().join("sikil_test_clean_stale");
        fs::create_dir_all(&temp_dir).ok();

        // Add entry for existing path
        let existing_entry = ScanEntry {
            path: temp_dir.clone(),
            mtime: 1234567890,
            size: 1024,
            content_hash: "existing".to_string(),
            cached_at: SqliteCache::now(),
            skill_name: Some("existing-skill".to_string()),
            is_valid_skill: true,
        };

        // Add entry for non-existent path
        let stale_entry = ScanEntry {
            path: PathBuf::from("/nonexistent/path/that/does/not/exist"),
            mtime: 1234567890,
            size: 1024,
            content_hash: "stale".to_string(),
            cached_at: SqliteCache::now(),
            skill_name: Some("stale-skill".to_string()),
            is_valid_skill: true,
        };

        cache.put(&existing_entry).unwrap();
        cache.put(&stale_entry).unwrap();

        // Clean stale entries
        let removed = cache.clean_stale().unwrap();
        assert_eq!(removed, 1);

        // Verify stale entry is removed (using get_raw)
        assert!(cache
            .get_raw(&PathBuf::from("/nonexistent/path/that/does/not/exist"))
            .unwrap()
            .is_none());

        // Verify existing entry is still there (using get_raw)
        assert!(cache.get_raw(&temp_dir).unwrap().is_some());

        // Clean up
        fs::remove_dir_all(temp_dir).ok();
    }

    #[test]
    fn test_cache_replace_existing_entry() {
        let cache = create_temp_cache().unwrap();

        let entry1 = ScanEntry {
            path: PathBuf::from("/test/skill"),
            mtime: 1234567890,
            size: 1024,
            content_hash: "hash1".to_string(),
            cached_at: SqliteCache::now(),
            skill_name: Some("skill-v1".to_string()),
            is_valid_skill: true,
        };

        let entry2 = ScanEntry {
            path: PathBuf::from("/test/skill"),
            mtime: 1234567891,
            size: 2048,
            content_hash: "hash2".to_string(),
            cached_at: SqliteCache::now(),
            skill_name: Some("skill-v2".to_string()),
            is_valid_skill: false,
        };

        cache.put(&entry1).unwrap();
        cache.put(&entry2).unwrap();

        let retrieved = cache
            .get_raw(&PathBuf::from("/test/skill"))
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.mtime, 1234567891);
        assert_eq!(retrieved.size, 2048);
        assert_eq!(retrieved.content_hash, "hash2");
        assert_eq!(retrieved.skill_name, Some("skill-v2".to_string()));
        assert!(!retrieved.is_valid_skill);
    }

    #[test]
    fn test_cache_entry_with_none_skill_name() {
        let cache = create_temp_cache().unwrap();

        let entry = ScanEntry {
            path: PathBuf::from("/test/invalid"),
            mtime: 1234567890,
            size: 512,
            content_hash: "no-skill".to_string(),
            cached_at: SqliteCache::now(),
            skill_name: None,
            is_valid_skill: false,
        };

        cache.put(&entry).unwrap();

        let retrieved = cache
            .get_raw(&PathBuf::from("/test/invalid"))
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.skill_name, None);
        assert!(!retrieved.is_valid_skill);
    }

    #[test]
    fn test_cache_put_rejects_oversized_hash() {
        let cache = create_temp_cache().unwrap();

        let entry = ScanEntry {
            path: PathBuf::from("/test/skill"),
            mtime: 1234567890,
            size: 1024,
            content_hash: "a".repeat(MAX_HASH_SIZE + 1),
            cached_at: SqliteCache::now(),
            skill_name: Some("test-skill".to_string()),
            is_valid_skill: true,
        };

        let result = cache.put(&entry);
        assert!(result.is_err());

        if let Err(SikilError::ValidationError { reason }) = result {
            assert!(reason.contains("content hash too long"));
        } else {
            panic!("Expected ValidationError");
        }
    }
}
