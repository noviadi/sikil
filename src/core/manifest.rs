use std::collections::BTreeMap;
use std::path::Path;

use fs_err as fs;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::core::errors::SikilError;
use crate::utils::atomic::atomic_write_file;

const MANIFEST_MAX_SIZE: u64 = 1_048_576; // 1 MB

// ---- Public types ----

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    #[serde(default, rename = "skill")]
    pub skills: BTreeMap<String, ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ManifestEntry {
    pub source: String,
    pub rev: Option<String>,
    pub subdir: Option<String>,
    pub agents: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Lockfile {
    pub version: u32,
    #[serde(default, rename = "skill")]
    pub skills: BTreeMap<String, LockfileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LockfileEntry {
    pub source: String,
    pub resolved_url: String,
    pub commit: String,
    pub content_hash: String,
    pub fetched_at: String,
}

// ---- Load / Save ----

pub fn load_manifest(path: &Path) -> Result<Manifest, SikilError> {
    if !path.exists() {
        return Err(SikilError::ManifestNotFound {
            path: path.to_path_buf(),
        });
    }

    let metadata = fs::metadata(path).map_err(|e| SikilError::ManifestParseError {
        path: path.to_path_buf(),
        reason: format!("failed to read file metadata: {}", e),
    })?;
    if metadata.len() > MANIFEST_MAX_SIZE {
        return Err(SikilError::ManifestParseError {
            path: path.to_path_buf(),
            reason: "manifest exceeds 1 MB".to_string(),
        });
    }

    let content = fs::read_to_string(path).map_err(|e| SikilError::ManifestParseError {
        path: path.to_path_buf(),
        reason: format!("failed to read file: {}", e),
    })?;

    let manifest: Manifest =
        toml::from_str(&content).map_err(|e| SikilError::ManifestParseError {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;

    if manifest.schema_version != 1 {
        return Err(SikilError::ManifestParseError {
            path: path.to_path_buf(),
            reason: format!(
                "unsupported schema_version: {} (expected 1)",
                manifest.schema_version
            ),
        });
    }

    Ok(manifest)
}

pub fn save_manifest(path: &Path, manifest: &Manifest) -> Result<(), SikilError> {
    let content = toml::to_string_pretty(manifest).map_err(|e| SikilError::ManifestParseError {
        path: path.to_path_buf(),
        reason: format!("failed to serialize manifest: {}", e),
    })?;
    atomic_write_file(path, content.as_bytes())
}

pub fn load_lockfile(path: &Path) -> Result<Lockfile, SikilError> {
    if !path.exists() {
        return Ok(Lockfile {
            version: 1,
            skills: BTreeMap::new(),
        });
    }

    let content = fs::read_to_string(path).map_err(|e| SikilError::LockfileMismatch {
        reason: format!("failed to read lockfile: {}", e),
    })?;

    let lockfile: Lockfile =
        toml::from_str(&content).map_err(|e| SikilError::LockfileMismatch {
            reason: e.to_string(),
        })?;

    if lockfile.version != 1 {
        return Err(SikilError::LockfileMismatch {
            reason: format!(
                "lockfile version {} is not supported (expected 1)",
                lockfile.version
            ),
        });
    }

    Ok(lockfile)
}

pub fn save_lockfile(path: &Path, lockfile: &Lockfile) -> Result<(), SikilError> {
    let content = toml::to_string_pretty(lockfile).map_err(|e| SikilError::LockfileMismatch {
        reason: format!("failed to serialize lockfile: {}", e),
    })?;
    atomic_write_file(path, content.as_bytes())
}

// ---- Content hash ----

/// Compute `sha256:<hex>` for the directory tree at `path`,
/// excluding `.git/` directories and `.sikil-source.toml` sidecar files.
pub fn compute_content_hash(path: &Path) -> Result<String, SikilError> {
    if !path.exists() {
        return Err(SikilError::DirectoryNotFound {
            path: path.to_path_buf(),
        });
    }

    if !path.is_dir() {
        return Err(SikilError::ValidationError {
            reason: format!("content hash target is not a directory: {}", path.display()),
        });
    }

    let mut files: Vec<(String, Vec<u8>)> = Vec::new();

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            // Skip .git directories and sidecar files
            name != ".git" && name != ".sikil-source.toml"
        })
    {
        let entry = entry.map_err(|e| SikilError::ValidationError {
            reason: format!("failed to read directory entry: {}", e),
        })?;

        if !entry.file_type().is_file() {
            continue;
        }

        let relative = entry
            .path()
            .strip_prefix(path)
            .map_err(|e| SikilError::PathTraversal {
                path: e.to_string(),
            })?;

        let content = fs::read(entry.path()).map_err(|e| SikilError::ValidationError {
            reason: format!("failed to read file {}: {}", relative.display(), e),
        })?;

        files.push((relative.to_string_lossy().to_string(), content));
    }

    // Sort by relative path for determinism
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let mut hasher = Sha256::new();
    for (rel_path, content) in &files {
        hasher.update(rel_path.as_bytes());
        hasher.update(b"\n");
        hasher.update(content);
    }

    let hash = hasher.finalize();
    Ok(format!("sha256:{:x}", hash))
}

// ---- Reconciliation ----

/// Action produced by comparing manifest entries against lockfile state.
#[derive(Debug, Clone, PartialEq)]
pub enum ReconcileAction {
    /// New or changed entry — needs fetching and vendoring.
    Resolve { name: String, entry: ManifestEntry },
    /// Orphaned lockfile entry — vendored bytes and symlinks should be removed.
    Remove { name: String, entry: LockfileEntry },
    /// Unchanged — vendored bytes exist and content hash matches.
    Keep { name: String },
}

/// Compare manifest entries against lockfile state and the vendored skill
/// directory, producing a list of actions.
///
/// - `skills_dir` is `<project_root>/.sikil/skills/`.
/// - An entry with no matching lockfile entry → `Resolve`.
/// - An entry whose vendored bytes are missing or whose on-disk content hash
///   differs from the lockfile → `Resolve`.
/// - An entry whose on-disk content hash matches the lockfile → `Keep`.
/// - A lockfile entry with no matching manifest entry → `Remove`.
pub fn reconcile(
    manifest: &Manifest,
    lockfile: &Lockfile,
    skills_dir: &Path,
) -> Vec<ReconcileAction> {
    let mut actions = Vec::new();

    // Process manifest entries
    for (name, entry) in &manifest.skills {
        if let Some(lock_entry) = lockfile.skills.get(name) {
            // Entry exists in lockfile — check if vendored bytes are still valid
            let vendored = skills_dir.join(name);
            if vendored.is_dir() {
                match compute_content_hash(&vendored) {
                    Ok(hash) if hash == lock_entry.content_hash => {
                        actions.push(ReconcileAction::Keep { name: name.clone() });
                        continue;
                    }
                    _ => {
                        // Hash mismatch or error — needs re-resolution
                    }
                }
            }
            // Missing vendored bytes or hash mismatch
            actions.push(ReconcileAction::Resolve {
                name: name.clone(),
                entry: entry.clone(),
            });
        } else {
            // No lockfile entry — fresh resolution required
            actions.push(ReconcileAction::Resolve {
                name: name.clone(),
                entry: entry.clone(),
            });
        }
    }

    // Orphaned lockfile entries (not in manifest)
    for (name, lock_entry) in &lockfile.skills {
        if !manifest.skills.contains_key(name) {
            actions.push(ReconcileAction::Remove {
                name: name.clone(),
                entry: lock_entry.clone(),
            });
        }
    }

    actions
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_manifest(dir: &Path, content: &str) -> PathBuf {
        let sikil_dir = dir.join(".sikil");
        fs::create_dir_all(&sikil_dir).unwrap();
        let path = sikil_dir.join("manifest.toml");
        fs::write(&path, content).unwrap();
        path
    }

    fn create_test_lockfile(dir: &Path, content: &str) -> PathBuf {
        let sikil_dir = dir.join(".sikil");
        fs::create_dir_all(&sikil_dir).unwrap();
        let path = sikil_dir.join("lock.toml");
        fs::write(&path, content).unwrap();
        path
    }

    // ---- Manifest load tests ----

    #[test]
    fn test_load_manifest_valid() {
        let tmp = TempDir::new().unwrap();
        let path = create_test_manifest(
            tmp.path(),
            r#"
schema_version = 1

[skill.my-skill]
source = "owner/repo"

[skill.another-skill]
source = "other/repo"
rev = "v2.0"
subdir = "skills/another"
agents = ["claude-code"]
"#,
        );

        let manifest = load_manifest(&path).unwrap();
        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.skills.len(), 2);

        let entry = &manifest.skills["my-skill"];
        assert_eq!(entry.source, "owner/repo");
        assert_eq!(entry.rev, None);
        assert_eq!(entry.subdir, None);
        assert_eq!(entry.agents, None);

        let entry2 = &manifest.skills["another-skill"];
        assert_eq!(entry2.source, "other/repo");
        assert_eq!(entry2.rev, Some("v2.0".to_string()));
        assert_eq!(entry2.subdir, Some("skills/another".to_string()));
        assert_eq!(entry2.agents, Some(vec!["claude-code".to_string()]));
    }

    #[test]
    fn test_load_manifest_missing_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.toml");

        let result = load_manifest(&path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SikilError::ManifestNotFound { .. }));
    }

    #[test]
    fn test_load_manifest_no_skills() {
        let tmp = TempDir::new().unwrap();
        let path = create_test_manifest(tmp.path(), "schema_version = 1\n");

        let manifest = load_manifest(&path).unwrap();
        assert_eq!(manifest.schema_version, 1);
        assert!(manifest.skills.is_empty());
    }

    #[test]
    fn test_load_manifest_unsupported_schema_version() {
        let tmp = TempDir::new().unwrap();
        let path = create_test_manifest(tmp.path(), "schema_version = 2\n");

        let result = load_manifest(&path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SikilError::ManifestParseError { .. }));
        assert!(err.to_string().contains("unsupported schema_version"));
    }

    #[test]
    fn test_load_manifest_unknown_top_level_fields() {
        let tmp = TempDir::new().unwrap();
        let path = create_test_manifest(
            tmp.path(),
            r#"
schema_version = 1
unknown_field = "not allowed"
"#,
        );

        let result = load_manifest(&path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SikilError::ManifestParseError { .. }));
    }

    #[test]
    fn test_load_manifest_unknown_entry_fields() {
        let tmp = TempDir::new().unwrap();
        let path = create_test_manifest(
            tmp.path(),
            r#"
schema_version = 1

[skill.my-skill]
source = "owner/repo"
unknown = "field"
"#,
        );

        let result = load_manifest(&path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SikilError::ManifestParseError { .. }));
    }

    #[test]
    fn test_load_manifest_exceeds_1mb() {
        let tmp = TempDir::new().unwrap();
        let sikil_dir = tmp.path().join(".sikil");
        fs::create_dir_all(&sikil_dir).unwrap();
        let path = sikil_dir.join("manifest.toml");

        let large_content = format!("schema_version = 1\n{}", "a".repeat(1_048_577));
        fs::write(&path, &large_content).unwrap();

        let result = load_manifest(&path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SikilError::ManifestParseError { .. }));
        assert!(err.to_string().contains("exceeds 1 MB"));
    }

    // ---- Manifest save tests ----

    #[test]
    fn test_save_and_load_manifest_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let sikil_dir = tmp.path().join(".sikil");
        fs::create_dir_all(&sikil_dir).unwrap();
        let path = sikil_dir.join("manifest.toml");

        let mut skills = BTreeMap::new();
        skills.insert(
            "my-skill".to_string(),
            ManifestEntry {
                source: "owner/repo".to_string(),
                rev: Some("main".to_string()),
                subdir: None,
                agents: Some(vec!["claude-code".to_string()]),
            },
        );
        skills.insert(
            "simple-skill".to_string(),
            ManifestEntry {
                source: "user/project".to_string(),
                rev: None,
                subdir: None,
                agents: None,
            },
        );

        let original = Manifest {
            schema_version: 1,
            skills,
        };

        save_manifest(&path, &original).unwrap();
        let loaded = load_manifest(&path).unwrap();

        assert_eq!(loaded, original);
    }

    #[test]
    fn test_save_manifest_atomic() {
        let tmp = TempDir::new().unwrap();
        let sikil_dir = tmp.path().join(".sikil");
        fs::create_dir_all(&sikil_dir).unwrap();
        let path = sikil_dir.join("manifest.toml");

        let manifest = Manifest {
            schema_version: 1,
            skills: BTreeMap::new(),
        };

        save_manifest(&path, &manifest).unwrap();
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("schema_version = 1"));

        // No orphaned temp files
        for entry in fs::read_dir(sikil_dir).unwrap() {
            let name = entry.unwrap().file_name();
            let name_str = name.to_string_lossy();
            assert!(
                !name_str.contains(".tmp."),
                "Found orphaned temp file: {}",
                name_str
            );
        }
    }

    // ---- Lockfile load tests ----

    #[test]
    fn test_load_lockfile_valid() {
        let tmp = TempDir::new().unwrap();
        let path = create_test_lockfile(
            tmp.path(),
            r#"
version = 1

[skill.my-skill]
source = "owner/repo"
resolved_url = "https://github.com/owner/repo.git"
commit = "abc123def456"
content_hash = "sha256:deadbeef"
fetched_at = "2024-01-01T00:00:00Z"
"#,
        );

        let lockfile = load_lockfile(&path).unwrap();
        assert_eq!(lockfile.version, 1);
        assert_eq!(lockfile.skills.len(), 1);

        let entry = &lockfile.skills["my-skill"];
        assert_eq!(entry.source, "owner/repo");
        assert_eq!(entry.resolved_url, "https://github.com/owner/repo.git");
        assert_eq!(entry.commit, "abc123def456");
        assert_eq!(entry.content_hash, "sha256:deadbeef");
        assert_eq!(entry.fetched_at, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_load_lockfile_missing_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nonexistent.toml");

        let lockfile = load_lockfile(&path).unwrap();
        assert_eq!(lockfile.version, 1);
        assert!(lockfile.skills.is_empty());
    }

    #[test]
    fn test_load_lockfile_unsupported_version() {
        let tmp = TempDir::new().unwrap();
        let path = create_test_lockfile(tmp.path(), "version = 2\n");

        let result = load_lockfile(&path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SikilError::LockfileMismatch { .. }));
        assert!(err.to_string().contains("not supported"));
    }

    #[test]
    fn test_load_lockfile_unknown_fields() {
        let tmp = TempDir::new().unwrap();
        let path = create_test_lockfile(
            tmp.path(),
            r#"
version = 1
unknown = "field"
"#,
        );

        let result = load_lockfile(&path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SikilError::LockfileMismatch { .. }
        ));
    }

    // ---- Lockfile save tests ----

    #[test]
    fn test_save_and_load_lockfile_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let sikil_dir = tmp.path().join(".sikil");
        fs::create_dir_all(&sikil_dir).unwrap();
        let path = sikil_dir.join("lock.toml");

        let mut skills = BTreeMap::new();
        skills.insert(
            "my-skill".to_string(),
            LockfileEntry {
                source: "owner/repo".to_string(),
                resolved_url: "https://github.com/owner/repo.git".to_string(),
                commit: "abc123".to_string(),
                content_hash: "sha256:deadbeef".to_string(),
                fetched_at: "2024-01-01T00:00:00Z".to_string(),
            },
        );

        let original = Lockfile { version: 1, skills };

        save_lockfile(&path, &original).unwrap();
        let loaded = load_lockfile(&path).unwrap();

        assert_eq!(loaded, original);
    }

    #[test]
    fn test_save_lockfile_atomic() {
        let tmp = TempDir::new().unwrap();
        let sikil_dir = tmp.path().join(".sikil");
        fs::create_dir_all(&sikil_dir).unwrap();
        let path = sikil_dir.join("lock.toml");

        let lockfile = Lockfile {
            version: 1,
            skills: BTreeMap::new(),
        };

        save_lockfile(&path, &lockfile).unwrap();
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("version = 1"));

        // No orphaned temp files
        for entry in fs::read_dir(sikil_dir).unwrap() {
            let name = entry.unwrap().file_name();
            let name_str = name.to_string_lossy();
            assert!(
                !name_str.contains(".tmp."),
                "Found orphaned temp file: {}",
                name_str
            );
        }
    }

    // ---- Content hash tests ----

    #[test]
    fn test_compute_content_hash_basic() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = tmp.path().join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Test Skill").unwrap();
        fs::write(skill_dir.join("script.sh"), "echo hello").unwrap();

        let hash = compute_content_hash(&skill_dir).unwrap();
        assert!(hash.starts_with("sha256:"));
        assert!(hash.len() > 10);
    }

    #[test]
    fn test_compute_content_hash_excludes_git() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = tmp.path().join("skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Skill").unwrap();

        // Create .git directory with content
        let git_dir = skill_dir.join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/main").unwrap();
        fs::write(git_dir.join("config"), "[core]").unwrap();

        let hash = compute_content_hash(&skill_dir).unwrap();

        // Remove .git and verify hash is the same (proving .git was excluded)
        fs::remove_dir_all(&git_dir).unwrap();
        let hash_without_git = compute_content_hash(&skill_dir).unwrap();

        assert_eq!(hash, hash_without_git);
    }

    #[test]
    fn test_compute_content_hash_excludes_sidecar() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = tmp.path().join("skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Skill").unwrap();

        // Compute without sidecar
        let hash_no_sidecar = compute_content_hash(&skill_dir).unwrap();

        // Add sidecar
        fs::write(
            skill_dir.join(".sikil-source.toml"),
            "version = 1\nsource = \"test\"",
        )
        .unwrap();

        let hash_with_sidecar = compute_content_hash(&skill_dir).unwrap();

        // Should be identical — sidecar is excluded
        assert_eq!(hash_no_sidecar, hash_with_sidecar);
    }

    #[test]
    fn test_compute_content_hash_deterministic() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = tmp.path().join("skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Skill").unwrap();
        fs::write(skill_dir.join("data.txt"), "some data").unwrap();

        let hash1 = compute_content_hash(&skill_dir).unwrap();
        let hash2 = compute_content_hash(&skill_dir).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_content_hash_differs_for_different_content() {
        let tmp1 = TempDir::new().unwrap();
        let skill_dir1 = tmp1.path().join("skill");
        fs::create_dir_all(&skill_dir1).unwrap();
        fs::write(skill_dir1.join("SKILL.md"), "# Skill A").unwrap();

        let tmp2 = TempDir::new().unwrap();
        let skill_dir2 = tmp2.path().join("skill");
        fs::create_dir_all(&skill_dir2).unwrap();
        fs::write(skill_dir2.join("SKILL.md"), "# Skill B").unwrap();

        let hash1 = compute_content_hash(&skill_dir1).unwrap();
        let hash2 = compute_content_hash(&skill_dir2).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_compute_content_hash_sha256_prefix() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = tmp.path().join("skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Skill").unwrap();

        let hash = compute_content_hash(&skill_dir).unwrap();

        // sha256: prefix + 64 hex characters
        assert!(hash.starts_with("sha256:"));
        let hex_part = hash.strip_prefix("sha256:").unwrap();
        assert_eq!(hex_part.len(), 64);
        assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_compute_content_hash_missing_path() {
        let result = compute_content_hash(Path::new("/nonexistent/path"));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SikilError::DirectoryNotFound { .. }
        ));
    }

    #[test]
    fn test_compute_content_hash_nested_directories() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = tmp.path().join("skill");
        fs::create_dir_all(skill_dir.join("scripts")).unwrap();
        fs::create_dir_all(skill_dir.join("references").join("deep")).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Skill").unwrap();
        fs::write(skill_dir.join("scripts").join("run.sh"), "echo run").unwrap();
        fs::write(
            skill_dir.join("references").join("deep").join("note.md"),
            "deep note",
        )
        .unwrap();

        let hash = compute_content_hash(&skill_dir).unwrap();
        assert!(hash.starts_with("sha256:"));
    }

    // ---- Manifest entry validation edge cases ----

    #[test]
    fn test_manifest_entry_with_all_optional_fields() {
        let tmp = TempDir::new().unwrap();
        let path = create_test_manifest(
            tmp.path(),
            r#"
schema_version = 1

[skill.full-skill]
source = "owner/repo"
rev = "v1.0"
subdir = "skills/full"
agents = ["claude-code", "windsurf"]
"#,
        );

        let manifest = load_manifest(&path).unwrap();
        let entry = &manifest.skills["full-skill"];
        assert_eq!(entry.source, "owner/repo");
        assert_eq!(entry.rev, Some("v1.0".to_string()));
        assert_eq!(entry.subdir, Some("skills/full".to_string()));
        assert_eq!(
            entry.agents,
            Some(vec!["claude-code".to_string(), "windsurf".to_string()])
        );
    }

    #[test]
    fn test_manifest_entry_with_only_required_fields() {
        let tmp = TempDir::new().unwrap();
        let path = create_test_manifest(
            tmp.path(),
            r#"
schema_version = 1

[skill.minimal-skill]
source = "owner/repo"
"#,
        );

        let manifest = load_manifest(&path).unwrap();
        let entry = &manifest.skills["minimal-skill"];
        assert_eq!(entry.source, "owner/repo");
        assert_eq!(entry.rev, None);
        assert_eq!(entry.subdir, None);
        assert_eq!(entry.agents, None);
    }

    // ---- Idempotency ----

    #[test]
    fn test_save_manifest_idempotent() {
        let tmp = TempDir::new().unwrap();
        let sikil_dir = tmp.path().join(".sikil");
        fs::create_dir_all(&sikil_dir).unwrap();
        let path = sikil_dir.join("manifest.toml");

        let mut skills = BTreeMap::new();
        skills.insert(
            "my-skill".to_string(),
            ManifestEntry {
                source: "owner/repo".to_string(),
                rev: None,
                subdir: None,
                agents: None,
            },
        );

        let manifest = Manifest {
            schema_version: 1,
            skills,
        };

        save_manifest(&path, &manifest).unwrap();
        let content1 = fs::read_to_string(&path).unwrap();

        // Save again with the same data
        save_manifest(&path, &manifest).unwrap();
        let content2 = fs::read_to_string(&path).unwrap();

        // Content should be identical
        assert_eq!(content1, content2);
    }

    // ---- Reconciliation tests ----

    fn make_manifest(skills: Vec<(&str, &str)>) -> Manifest {
        Manifest {
            schema_version: 1,
            skills: skills
                .into_iter()
                .map(|(name, source)| {
                    (
                        name.to_string(),
                        ManifestEntry {
                            source: source.to_string(),
                            rev: None,
                            subdir: None,
                            agents: None,
                        },
                    )
                })
                .collect(),
        }
    }

    fn make_lockfile(skills_dir: &Path, skills: Vec<(&str, &str, &str)>) -> Lockfile {
        let mut entries = BTreeMap::new();
        for (name, source, content) in &skills {
            let skill_dir = skills_dir.join(name);
            fs::create_dir_all(&skill_dir).unwrap();
            fs::write(skill_dir.join("SKILL.md"), content.as_bytes()).unwrap();

            let hash = compute_content_hash(&skill_dir).unwrap();
            entries.insert(
                name.to_string(),
                LockfileEntry {
                    source: source.to_string(),
                    resolved_url: format!("https://github.com/{}.git", source),
                    commit: "abc123".to_string(),
                    content_hash: hash,
                    fetched_at: "2024-01-01T00:00:00Z".to_string(),
                },
            );
        }
        Lockfile {
            version: 1,
            skills: entries,
        }
    }

    #[test]
    fn test_reconcile_new_manifest_entry_triggers_resolve() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let manifest = make_manifest(vec![("new-skill", "owner/repo")]);
        let lockfile = Lockfile {
            version: 1,
            skills: BTreeMap::new(),
        };

        let actions = reconcile(&manifest, &lockfile, &skills_dir);
        assert_eq!(actions.len(), 1);
        assert_eq!(
            actions[0],
            ReconcileAction::Resolve {
                name: "new-skill".to_string(),
                entry: ManifestEntry {
                    source: "owner/repo".to_string(),
                    rev: None,
                    subdir: None,
                    agents: None,
                }
            }
        );
    }

    #[test]
    fn test_reconcile_orphaned_lockfile_entry_triggers_remove() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let manifest = make_manifest(vec![]);
        let lockfile = make_lockfile(
            &skills_dir,
            vec![("orphan-skill", "owner/orphan", "# Orphan")],
        );

        let actions = reconcile(&manifest, &lockfile, &skills_dir);
        assert_eq!(actions.len(), 1);
        assert!(
            matches!(&actions[0], ReconcileAction::Remove { name, .. } if name == "orphan-skill")
        );
    }

    #[test]
    fn test_reconcile_matching_entry_with_valid_content_triggers_keep() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let manifest = make_manifest(vec![("my-skill", "owner/repo")]);
        let lockfile = make_lockfile(
            &skills_dir,
            vec![("my-skill", "owner/repo", "# Skill content")],
        );

        let actions = reconcile(&manifest, &lockfile, &skills_dir);
        assert_eq!(actions.len(), 1);
        assert!(matches!(&actions[0], ReconcileAction::Keep { name } if name == "my-skill"));
    }

    #[test]
    fn test_reconcile_matching_entry_with_missing_vendored_bytes_triggers_resolve() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let manifest = make_manifest(vec![("my-skill", "owner/repo")]);

        // Lockfile entry exists but vendored bytes don't
        let mut lock_skills = BTreeMap::new();
        lock_skills.insert(
            "my-skill".to_string(),
            LockfileEntry {
                source: "owner/repo".to_string(),
                resolved_url: "https://github.com/owner/repo.git".to_string(),
                commit: "abc123".to_string(),
                content_hash: "sha256:deadbeef".to_string(),
                fetched_at: "2024-01-01T00:00:00Z".to_string(),
            },
        );
        let lockfile = Lockfile {
            version: 1,
            skills: lock_skills,
        };

        let actions = reconcile(&manifest, &lockfile, &skills_dir);
        assert_eq!(actions.len(), 1);
        assert!(matches!(&actions[0], ReconcileAction::Resolve { name, .. } if name == "my-skill"));
    }

    #[test]
    fn test_reconcile_matching_entry_with_changed_content_triggers_resolve() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let manifest = make_manifest(vec![("my-skill", "owner/repo")]);

        // Create vendored bytes
        let skill_dir = skills_dir.join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Changed content").unwrap();

        // Lockfile entry with wrong hash
        let mut lock_skills = BTreeMap::new();
        lock_skills.insert(
            "my-skill".to_string(),
            LockfileEntry {
                source: "owner/repo".to_string(),
                resolved_url: "https://github.com/owner/repo.git".to_string(),
                commit: "abc123".to_string(),
                content_hash: "sha256:0000000000000000".to_string(),
                fetched_at: "2024-01-01T00:00:00Z".to_string(),
            },
        );
        let lockfile = Lockfile {
            version: 1,
            skills: lock_skills,
        };

        let actions = reconcile(&manifest, &lockfile, &skills_dir);
        assert_eq!(actions.len(), 1);
        assert!(matches!(&actions[0], ReconcileAction::Resolve { name, .. } if name == "my-skill"));
    }

    #[test]
    fn test_reconcile_mixed_actions() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let manifest = make_manifest(vec![
            ("keep-skill", "owner/keep"),
            ("resolve-skill", "owner/resolve"),
        ]);

        // Create vendored bytes for keep-skill only
        let lockfile = make_lockfile(
            &skills_dir,
            vec![
                ("keep-skill", "owner/keep", "# Keep content"),
                ("remove-skill", "owner/remove", "# Remove content"),
            ],
        );

        let actions = reconcile(&manifest, &lockfile, &skills_dir);

        let keep_count = actions
            .iter()
            .filter(|a| matches!(a, ReconcileAction::Keep { .. }))
            .count();
        let resolve_count = actions
            .iter()
            .filter(|a| matches!(a, ReconcileAction::Resolve { .. }))
            .count();
        let remove_count = actions
            .iter()
            .filter(|a| matches!(a, ReconcileAction::Remove { .. }))
            .count();

        assert_eq!(keep_count, 1);
        assert_eq!(resolve_count, 1);
        assert_eq!(remove_count, 1);
    }

    #[test]
    fn test_reconcile_empty_manifest_removes_all_lockfile_entries() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let manifest = make_manifest(vec![]);
        let lockfile = make_lockfile(
            &skills_dir,
            vec![("skill-a", "owner/a", "# A"), ("skill-b", "owner/b", "# B")],
        );

        let actions = reconcile(&manifest, &lockfile, &skills_dir);
        assert_eq!(actions.len(), 2);
        assert!(actions
            .iter()
            .all(|a| matches!(a, ReconcileAction::Remove { .. })));
    }

    #[test]
    fn test_reconcile_empty_lockfile_resolves_all_manifest_entries() {
        let tmp = TempDir::new().unwrap();
        let skills_dir = tmp.path().join("skills");
        fs::create_dir_all(&skills_dir).unwrap();

        let manifest = make_manifest(vec![("skill-a", "owner/a"), ("skill-b", "owner/b")]);
        let lockfile = Lockfile {
            version: 1,
            skills: BTreeMap::new(),
        };

        let actions = reconcile(&manifest, &lockfile, &skills_dir);
        assert_eq!(actions.len(), 2);
        assert!(actions
            .iter()
            .all(|a| matches!(a, ReconcileAction::Resolve { .. })));
    }
}
