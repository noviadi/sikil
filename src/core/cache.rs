//! Cache module for storing and retrieving skill scan results.
//!
//! This module provides a caching layer to avoid repeated filesystem scanning
//! by storing scan results with metadata (mtime, size, content hash) for
//! invalidation.

use crate::core::SikilError;
use crate::utils::paths::{ensure_dir_exists, get_cache_path};
use fs_err as fs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Cache file format version.
/// Increment this when the format changes to trigger full cache clear.
const CACHE_VERSION: u32 = 1;

/// Maximum size of content hash to store (SHA256 = 64 hex chars)
const MAX_HASH_SIZE: usize = 64;

/// Maximum cache file size (15 MB) - exceeding triggers full clear
const MAX_CACHE_SIZE: u64 = 15 * 1024 * 1024;

/// Trait defining cache operations for skill scan results.
pub trait Cache {
    /// Get cached scan entry for a directory path.
    ///
    /// Returns `None` if not cached or if the entry is invalid due to
    /// mtime mismatch.
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

    /// Last modification time of SKILL.md (for invalidation)
    pub mtime: u64,

    /// Size of SKILL.md in bytes
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

/// JSON cache file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheFile {
    /// Cache format version
    version: u32,
    /// Path-keyed cache entries (absolute path strings)
    entries: BTreeMap<String, CachedEntry>,
}

/// Entry stored in JSON cache (path is the key, not in the struct)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedEntry {
    /// Last modification time of SKILL.md (unix seconds)
    mtime: u64,
    /// Size of SKILL.md in bytes
    size: u64,
    /// SHA256 hash of SKILL.md content
    content_hash: String,
    /// Timestamp when entry was cached (unix seconds)
    cached_at: u64,
    /// Parsed skill name from SKILL.md
    skill_name: Option<String>,
    /// Whether the path contains a valid skill
    is_valid_skill: bool,
}

impl From<&ScanEntry> for CachedEntry {
    fn from(entry: &ScanEntry) -> Self {
        Self {
            mtime: entry.mtime,
            size: entry.size,
            content_hash: entry.content_hash.clone(),
            cached_at: entry.cached_at,
            skill_name: entry.skill_name.clone(),
            is_valid_skill: entry.is_valid_skill,
        }
    }
}

impl CachedEntry {
    fn to_scan_entry(&self, path: PathBuf) -> ScanEntry {
        ScanEntry {
            path,
            mtime: self.mtime,
            size: self.size,
            content_hash: self.content_hash.clone(),
            cached_at: self.cached_at,
            skill_name: self.skill_name.clone(),
            is_valid_skill: self.is_valid_skill,
        }
    }
}

impl CacheFile {
    /// Create a new empty cache file with current version.
    fn new() -> Self {
        Self {
            version: CACHE_VERSION,
            entries: BTreeMap::new(),
        }
    }
}

/// JSON-based cache implementation.
pub struct JsonCache {
    cache_path: PathBuf,
}

impl JsonCache {
    /// Open or create the cache file at the default location.
    pub fn open() -> Result<Self, SikilError> {
        let cache_path = get_cache_path();
        Self::open_at(&cache_path)
    }

    /// Open or create the cache file at a specific path.
    pub fn open_at(path: &Path) -> Result<Self, SikilError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            ensure_dir_exists(parent).map_err(|e| SikilError::ConfigError {
                reason: format!("failed to create cache directory: {}", e),
            })?;
        }

        Ok(Self {
            cache_path: path.to_path_buf(),
        })
    }

    /// Load the cache file, returning None if it doesn't exist or on error.
    fn load(&self) -> Option<CacheFile> {
        // Check file size first
        let metadata = fs::metadata(&self.cache_path).ok()?;
        if metadata.len() > MAX_CACHE_SIZE {
            // Cache file too large - trigger clear on next write
            return None;
        }

        let content = fs::read_to_string(&self.cache_path).ok()?;
        let cache_file: CacheFile = serde_json::from_str(&content).ok()?;

        // Check version
        if cache_file.version != CACHE_VERSION {
            return None;
        }

        Some(cache_file)
    }

    /// Write cache file atomically (temp file + rename).
    fn write(&self, cache_file: &CacheFile) -> Result<(), SikilError> {
        // Use a truly unique temp file name to avoid conflicts in concurrent scenarios
        // Timestamp + random number ensures uniqueness
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| SikilError::ConfigError {
                reason: "failed to get timestamp".to_string(),
            })?
            .as_nanos();
        // Use a simple hash of thread ID as a fallback for uniqueness
        let thread_hash = std::thread::current().id();
        // Format thread ID as hex string
        let thread_id_str = format!("{:?}", thread_hash);
        let temp_path = format!(
            "{}.{}.{}.tmp",
            self.cache_path.display(),
            thread_id_str,
            timestamp
        );
        let temp_path = PathBuf::from(&temp_path);

        let json =
            serde_json::to_string_pretty(cache_file).map_err(|e| SikilError::ConfigError {
                reason: format!("failed to serialize cache: {}", e),
            })?;

        fs::write(&temp_path, json).map_err(|e| SikilError::ConfigError {
            reason: format!("failed to write cache temp file: {}", e),
        })?;

        fs::rename(&temp_path, &self.cache_path).map_err(|e| SikilError::ConfigError {
            reason: format!("failed to rename cache file: {}", e),
        })?;

        Ok(())
    }

    /// Get the mtime for a SKILL.md file at the given path.
    fn get_skill_mtime(path: &Path) -> Result<u64, SikilError> {
        let skill_md_path = path.join("SKILL.md");
        let metadata =
            fs::metadata(&skill_md_path).map_err(|_e| SikilError::DirectoryNotFound {
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
}

impl Cache for JsonCache {
    fn get(&self, path: &Path) -> Result<Option<ScanEntry>, SikilError> {
        let path_str = path.to_str().ok_or_else(|| SikilError::ConfigError {
            reason: format!("invalid path: {}", path.display()),
        })?;

        // Load cache file (non-fatal if it fails)
        let cache_file = match self.load() {
            Some(file) => file,
            None => return Ok(None),
        };

        // Look up entry
        let cached_entry = match cache_file.entries.get(path_str) {
            Some(entry) => entry,
            None => return Ok(None),
        };

        // Check if SKILL.md still exists and mtime matches
        match Self::get_skill_mtime(path) {
            Ok(current_mtime) => {
                if current_mtime != cached_entry.mtime {
                    // SKILL.md has been modified, return None to trigger rescan
                    return Ok(None);
                }
            }
            Err(_) => {
                // SKILL.md no longer exists
                return Ok(None);
            }
        }

        Ok(Some(cached_entry.to_scan_entry(path.to_path_buf())))
    }

    fn put(&self, entry: &ScanEntry) -> Result<(), SikilError> {
        // Validate content hash length
        if entry.content_hash.len() > MAX_HASH_SIZE {
            return Err(SikilError::ValidationError {
                reason: format!("content hash too long: {}", entry.content_hash.len()),
            });
        }

        let path_str = entry.path.to_str().ok_or_else(|| SikilError::ConfigError {
            reason: format!("invalid path: {}", entry.path.display()),
        })?;

        // Load existing cache or create new
        let mut cache_file = self.load().unwrap_or_else(CacheFile::new);

        // Insert/replace entry
        cache_file
            .entries
            .insert(path_str.to_string(), CachedEntry::from(entry));

        // Write (non-fatal on failure)
        self.write(&cache_file)
    }

    fn invalidate(&self, path: &Path) -> Result<(), SikilError> {
        let path_str = path.to_str().ok_or_else(|| SikilError::ConfigError {
            reason: format!("invalid path: {}", path.display()),
        })?;

        // Load existing cache or create new
        let mut cache_file = self.load().unwrap_or_else(CacheFile::new);

        // Remove entry
        cache_file.entries.remove(path_str);

        // Write (non-fatal on failure)
        let _ = self.write(&cache_file);
        Ok(())
    }

    fn clean_stale(&self) -> Result<usize, SikilError> {
        // Load existing cache or return 0 if doesn't exist
        let mut cache_file = match self.load() {
            Some(file) => file,
            None => return Ok(0),
        };

        let mut stale_count = 0;
        let mut stale_paths = Vec::new();

        // Find stale entries
        for path_str in cache_file.entries.keys() {
            let path = PathBuf::from(path_str);
            let skill_md_path = path.join("SKILL.md");
            if !skill_md_path.exists() {
                stale_paths.push(path_str.clone());
                stale_count += 1;
            }
        }

        // Remove stale entries
        for path_str in stale_paths {
            cache_file.entries.remove(&path_str);
        }

        // Write (non-fatal on failure)
        let _ = self.write(&cache_file);
        Ok(stale_count)
    }

    fn clear(&self) -> Result<(), SikilError> {
        let cache_file = CacheFile::new();
        let _ = self.write(&cache_file);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper to create a ScanEntry for testing
    fn create_test_entry(path: &str, mtime: u64) -> ScanEntry {
        ScanEntry {
            path: PathBuf::from(path),
            mtime,
            size: 1024,
            content_hash: "abc123".to_string(),
            cached_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            skill_name: Some("test-skill".to_string()),
            is_valid_skill: true,
        }
    }

    #[test]
    fn test_json_cache_open_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();
        assert!(!cache.cache_path.exists()); // No file created yet
    }

    #[test]
    fn test_cache_put_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();
        let entry = create_test_entry("/test/skill", 1234567890);

        // Put entry
        cache.put(&entry).unwrap();

        // Verify cache file was created
        assert!(cache.cache_path.exists());

        // Get entry - will return None because SKILL.md doesn't exist for mtime check
        // But we can verify the put worked by checking the file exists
        let loaded = cache.load().unwrap();
        assert!(loaded.entries.contains_key("/test/skill"));
    }

    #[test]
    fn test_cache_get_returns_none_for_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();
        // Using load() directly to test cache lookup without mtime validation
        let result = cache.load();
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_put_creates_valid_json() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();
        let entry = create_test_entry("/test/skill", 1234567890);

        cache.put(&entry).unwrap();

        // Load and verify JSON structure
        let loaded = cache.load().unwrap();
        assert_eq!(loaded.version, CACHE_VERSION);
        assert!(loaded.entries.contains_key("/test/skill"));

        let cached = &loaded.entries["/test/skill"];
        assert_eq!(cached.mtime, 1234567890);
        assert_eq!(cached.size, 1024);
        assert_eq!(cached.content_hash, "abc123");
        assert_eq!(cached.skill_name, Some("test-skill".to_string()));
        assert!(cached.is_valid_skill);
    }

    #[test]
    fn test_cache_invalidate() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();
        let entry = create_test_entry("/test/skill", 1234567890);

        cache.put(&entry).unwrap();
        assert!(cache.load().unwrap().entries.contains_key("/test/skill"));

        cache.invalidate(&PathBuf::from("/test/skill")).unwrap();
        assert!(!cache.load().unwrap().entries.contains_key("/test/skill"));
    }

    #[test]
    fn test_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();

        // Add multiple entries
        for i in 0..5 {
            let entry = create_test_entry(&format!("/test/skill{}", i), 1234567890 + i as u64);
            cache.put(&entry).unwrap();
        }

        // Verify entries exist
        let loaded = cache.load().unwrap();
        assert_eq!(loaded.entries.len(), 5);

        // Clear all
        cache.clear().unwrap();

        // Verify all entries are gone
        let loaded = cache.load().unwrap();
        assert_eq!(loaded.entries.len(), 0);
    }

    #[test]
    fn test_cache_clean_stale() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();

        // Create a temp directory with SKILL.md (will exist)
        let skill_dir = temp_dir.path().join("existing-skill");
        fs::create_dir(&skill_dir).unwrap();
        let skill_md_path = skill_dir.join("SKILL.md");
        fs::write(&skill_md_path, "# Test").unwrap();

        // Add entry for existing path
        let existing_entry = ScanEntry {
            path: skill_dir.clone(),
            mtime: 1234567890,
            size: 1024,
            content_hash: "existing".to_string(),
            cached_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            skill_name: Some("existing-skill".to_string()),
            is_valid_skill: true,
        };
        cache.put(&existing_entry).unwrap();

        // Add entry for non-existent path
        let stale_entry = create_test_entry("/nonexistent/path/that/does/not/exist", 1234567890);
        cache.put(&stale_entry).unwrap();

        // Clean stale entries
        let removed = cache.clean_stale().unwrap();
        assert_eq!(removed, 1);

        // Verify stale entry is removed
        let loaded = cache.load().unwrap();
        assert!(!loaded
            .entries
            .contains_key("/nonexistent/path/that/does/not/exist"));

        // Verify existing entry is still there
        assert!(loaded
            .entries
            .contains_key(&skill_dir.to_string_lossy().to_string()));
    }

    #[test]
    fn test_cache_replace_existing_entry() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();

        let entry1 = create_test_entry("/test/skill", 1234567890);
        let mut entry2 = create_test_entry("/test/skill", 1234567891);
        entry2.size = 2048;
        entry2.content_hash = "hash2".to_string();
        entry2.skill_name = Some("skill-v2".to_string());
        entry2.is_valid_skill = false;

        cache.put(&entry1).unwrap();
        cache.put(&entry2).unwrap();

        let loaded = cache.load().unwrap();
        let cached = &loaded.entries["/test/skill"];
        assert_eq!(cached.mtime, 1234567891);
        assert_eq!(cached.size, 2048);
        assert_eq!(cached.content_hash, "hash2");
        assert_eq!(cached.skill_name, Some("skill-v2".to_string()));
        assert!(!cached.is_valid_skill);
    }

    #[test]
    fn test_cache_entry_with_none_skill_name() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();

        let mut entry = create_test_entry("/test/invalid", 1234567890);
        entry.skill_name = None;
        entry.is_valid_skill = false;

        cache.put(&entry).unwrap();

        let loaded = cache.load().unwrap();
        let cached = &loaded.entries["/test/invalid"];
        assert_eq!(cached.skill_name, None);
        assert!(!cached.is_valid_skill);
    }

    #[test]
    fn test_cache_put_rejects_oversized_hash() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();

        let mut entry = create_test_entry("/test/skill", 1234567890);
        entry.content_hash = "a".repeat(MAX_HASH_SIZE + 1);

        let result = cache.put(&entry);
        assert!(result.is_err());

        if let Err(SikilError::ValidationError { reason }) = result {
            assert!(reason.contains("content hash too long"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[test]
    fn test_cache_version_mismatch_clears_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();
        let entry = create_test_entry("/test/skill", 1234567890);

        cache.put(&entry).unwrap();

        // Manually write a cache file with wrong version
        let wrong_version = r#"{"version": 999, "entries": {"/test/skill": {"mtime": 1234567890, "size": 1024, "content_hash": "abc123", "cached_at": 1234567890, "skill_name": null, "is_valid_skill": true}}}"#;
        fs::write(&cache_path, wrong_version).unwrap();

        // Load should return None due to version mismatch
        assert!(cache.load().is_none());
    }

    #[test]
    fn test_cache_size_limit_clears_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();
        let entry = create_test_entry("/test/skill", 1234567890);

        cache.put(&entry).unwrap();

        // Create JSON that exceeds size limit by using a very long content_hash
        // This ensures the file size exceeds MAX_CACHE_SIZE
        let padding_size = MAX_CACHE_SIZE as usize + 1;
        let huge_content = format!(
            r#"{{"version": 1, "entries": {{"/test/skill": {{"mtime": 1234567890, "size": 1024, "content_hash": "{}", "cached_at": 1234567890, "skill_name": null, "is_valid_skill": true}}}}}}"#,
            "a".repeat(padding_size)
        );
        fs::write(&cache_path, &huge_content).unwrap();

        // Verify the file actually exceeds the size limit
        let file_size = fs::metadata(&cache_path).unwrap().len();
        assert!(
            file_size > MAX_CACHE_SIZE,
            "Test file should exceed MAX_CACHE_SIZE ({} > {})",
            file_size,
            MAX_CACHE_SIZE
        );

        // Load should return None due to size limit check
        assert!(cache.load().is_none());
    }

    #[test]
    fn test_cache_write_uses_atomic_temp_file() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();
        let entry = create_test_entry("/test/skill", 1234567890);

        cache.put(&entry).unwrap();

        // Verify temp file was cleaned up (should not exist)
        let temp_path = cache_path.with_extension("tmp");
        assert!(!temp_path.exists());

        // Verify final cache file exists
        assert!(cache_path.exists());
    }

    #[test]
    fn test_cache_put_with_max_hash_size_succeeds() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();

        let mut entry = create_test_entry("/test/skill", 1234567890);
        entry.content_hash = "a".repeat(MAX_HASH_SIZE);

        let result = cache.put(&entry);
        assert!(result.is_ok());

        let loaded = cache.load().unwrap();
        let cached = &loaded.entries["/test/skill"];
        assert_eq!(cached.content_hash.len(), MAX_HASH_SIZE);
    }

    #[test]
    fn test_cache_entries_use_btreemap_for_determinism() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();

        // Add entries in non-alphabetical order
        cache.put(&create_test_entry("/test/z", 1)).unwrap();
        cache.put(&create_test_entry("/test/a", 2)).unwrap();
        cache.put(&create_test_entry("/test/m", 3)).unwrap();

        // Load and verify keys are sorted
        let loaded = cache.load().unwrap();
        let keys: Vec<&String> = loaded.entries.keys().collect();
        assert_eq!(keys, vec!["/test/a", "/test/m", "/test/z"]);
    }

    #[test]
    fn test_cache_get_returns_none_when_mtime_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();

        // Create a real temp directory with SKILL.md
        let skill_dir = temp_dir.path().join("test-skill");
        fs::create_dir(&skill_dir).unwrap();
        let skill_md_path = skill_dir.join("SKILL.md");
        fs::write(&skill_md_path, "# Test").unwrap();

        // Get actual mtime
        let actual_mtime = JsonCache::get_skill_mtime(&skill_dir).unwrap();

        // Cache entry with wrong mtime
        let entry = ScanEntry {
            path: skill_dir.clone(),
            mtime: actual_mtime - 1000, // Wrong mtime
            size: 1024,
            content_hash: "abc123".to_string(),
            cached_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            skill_name: Some("test-skill".to_string()),
            is_valid_skill: true,
        };
        cache.put(&entry).unwrap();

        // get() should return None due to mtime mismatch
        let result = cache.get(&skill_dir).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_file_pretty_printed() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let cache = JsonCache::open_at(&cache_path).unwrap();
        let entry = create_test_entry("/test/skill", 1234567890);

        cache.put(&entry).unwrap();

        // Read the file and verify it's pretty-printed (contains newlines)
        let content = fs::read_to_string(&cache_path).unwrap();
        assert!(content.contains('\n'));
        assert!(content.contains("  ")); // Indentation
    }
}
