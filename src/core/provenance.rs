use std::path::{Path, PathBuf};

use fs_err as fs;
use serde::{Deserialize, Serialize};

use crate::core::errors::SikilError;
use crate::utils::atomic::atomic_write_file;

const SIDECAR_FILENAME: &str = ".sikil-source.toml";
const SUPPORTED_SIDECAR_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceSidecar {
    pub version: u32,
    pub source: String,
    pub resolved_url: String,
    pub commit: String,
    pub content_hash: String,
    pub installed_at: String,
    pub installer: String,
}

pub fn sidecar_path(skill_dir: &Path) -> PathBuf {
    skill_dir.join(SIDECAR_FILENAME)
}

pub fn load_sidecar(skill_dir: &Path) -> Result<Option<ProvenanceSidecar>, SikilError> {
    let path = sidecar_path(skill_dir);
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path).map_err(|e| SikilError::ValidationError {
        reason: format!("failed to read provenance sidecar: {}", e),
    })?;

    let sidecar: ProvenanceSidecar =
        toml::from_str(&content).map_err(|e| SikilError::ValidationError {
            reason: format!("invalid provenance sidecar: {}", e),
        })?;

    if sidecar.version != SUPPORTED_SIDECAR_VERSION {
        return Err(SikilError::ValidationError {
            reason: format!(
                "unsupported sidecar version: {} (expected {})",
                sidecar.version, SUPPORTED_SIDECAR_VERSION
            ),
        });
    }

    Ok(Some(sidecar))
}

pub fn save_sidecar(skill_dir: &Path, sidecar: &ProvenanceSidecar) -> Result<(), SikilError> {
    let path = sidecar_path(skill_dir);
    let content = toml::to_string_pretty(sidecar).map_err(|e| SikilError::ValidationError {
        reason: format!("failed to serialize provenance sidecar: {}", e),
    })?;
    atomic_write_file(&path, content.as_bytes())
}

pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

pub fn installer_version() -> String {
    format!("sikil/{}", env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_skill_dir(dir: &Path, name: &str) -> PathBuf {
        let skill_dir = dir.join(name);
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# Test Skill").unwrap();
        skill_dir
    }

    fn write_sidecar(skill_dir: &Path, content: &str) {
        fs::write(sidecar_path(skill_dir), content).unwrap();
    }

    fn make_sidecar() -> ProvenanceSidecar {
        ProvenanceSidecar {
            version: 1,
            source: "owner/repo".to_string(),
            resolved_url: "https://github.com/owner/repo.git".to_string(),
            commit: "abc123def456789012345678901234567890abcd".to_string(),
            content_hash: "sha256:deadbeef12345678".to_string(),
            installed_at: "2024-01-15T10:30:00Z".to_string(),
            installer: "sikil/0.1.0".to_string(),
        }
    }

    // ---- Load tests ----

    #[test]
    fn test_load_sidecar_valid() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = create_test_skill_dir(tmp.path(), "my-skill");

        write_sidecar(
            &skill_dir,
            r#"
version = 1
source = "owner/repo"
resolved_url = "https://github.com/owner/repo.git"
commit = "abc123def456789012345678901234567890abcd"
content_hash = "sha256:deadbeef12345678"
installed_at = "2024-01-15T10:30:00Z"
installer = "sikil/0.1.0"
"#,
        );

        let result = load_sidecar(&skill_dir).unwrap();
        assert!(result.is_some());
        let sidecar = result.unwrap();
        assert_eq!(sidecar.version, 1);
        assert_eq!(sidecar.source, "owner/repo");
        assert_eq!(sidecar.resolved_url, "https://github.com/owner/repo.git");
        assert_eq!(sidecar.commit, "abc123def456789012345678901234567890abcd");
        assert_eq!(sidecar.content_hash, "sha256:deadbeef12345678");
        assert_eq!(sidecar.installed_at, "2024-01-15T10:30:00Z");
        assert_eq!(sidecar.installer, "sikil/0.1.0");
    }

    #[test]
    fn test_load_sidecar_missing_returns_none() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = create_test_skill_dir(tmp.path(), "my-skill");

        let result = load_sidecar(&skill_dir).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_load_sidecar_unsupported_version() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = create_test_skill_dir(tmp.path(), "my-skill");

        write_sidecar(
            &skill_dir,
            r#"
version = 2
source = "owner/repo"
resolved_url = "https://github.com/owner/repo.git"
commit = "abc123"
content_hash = "sha256:deadbeef"
installed_at = "2024-01-15T10:30:00Z"
installer = "sikil/0.1.0"
"#,
        );

        let result = load_sidecar(&skill_dir);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SikilError::ValidationError { .. }));
        assert!(
            err.to_string().contains("unsupported sidecar version"),
            "Expected 'unsupported sidecar version' in error, got: {}",
            err
        );
    }

    #[test]
    fn test_load_sidecar_unknown_fields() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = create_test_skill_dir(tmp.path(), "my-skill");

        write_sidecar(
            &skill_dir,
            r#"
version = 1
source = "owner/repo"
resolved_url = "https://github.com/owner/repo.git"
commit = "abc123"
content_hash = "sha256:deadbeef"
installed_at = "2024-01-15T10:30:00Z"
installer = "sikil/0.1.0"
unknown_field = "not allowed"
"#,
        );

        let result = load_sidecar(&skill_dir);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SikilError::ValidationError { .. }));
    }

    #[test]
    fn test_load_sidecar_invalid_toml() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = create_test_skill_dir(tmp.path(), "my-skill");

        write_sidecar(&skill_dir, "this is not valid toml {{{{");

        let result = load_sidecar(&skill_dir);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, SikilError::ValidationError { .. }));
    }

    // ---- Save tests ----

    #[test]
    fn test_save_and_load_sidecar_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = create_test_skill_dir(tmp.path(), "my-skill");

        let original = make_sidecar();
        save_sidecar(&skill_dir, &original).unwrap();

        let loaded = load_sidecar(&skill_dir).unwrap().unwrap();
        assert_eq!(loaded, original);
    }

    #[test]
    fn test_save_sidecar_atomic() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = create_test_skill_dir(tmp.path(), "my-skill");

        let sidecar = make_sidecar();
        save_sidecar(&skill_dir, &sidecar).unwrap();

        assert!(sidecar_path(&skill_dir).exists());

        // No orphaned temp files
        for entry in fs::read_dir(&skill_dir).unwrap() {
            let name = entry.unwrap().file_name();
            let name_str = name.to_string_lossy();
            assert!(
                !name_str.contains(".tmp."),
                "Found orphaned temp file: {}",
                name_str
            );
        }
    }

    #[test]
    fn test_save_sidecar_overwrites_existing() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = create_test_skill_dir(tmp.path(), "my-skill");

        let sidecar_v1 = ProvenanceSidecar {
            version: 1,
            source: "owner/repo".to_string(),
            resolved_url: "https://github.com/owner/repo.git".to_string(),
            commit: "aaa111".to_string(),
            content_hash: "sha256:hash1".to_string(),
            installed_at: "2024-01-01T00:00:00Z".to_string(),
            installer: "sikil/0.1.0".to_string(),
        };

        save_sidecar(&skill_dir, &sidecar_v1).unwrap();
        let loaded_v1 = load_sidecar(&skill_dir).unwrap().unwrap();
        assert_eq!(loaded_v1.commit, "aaa111");

        let sidecar_v2 = ProvenanceSidecar {
            commit: "bbb222".to_string(),
            ..sidecar_v1
        };

        save_sidecar(&skill_dir, &sidecar_v2).unwrap();
        let loaded_v2 = load_sidecar(&skill_dir).unwrap().unwrap();
        assert_eq!(loaded_v2.commit, "bbb222");
    }

    // ---- Sidecar path helper ----

    #[test]
    fn test_sidecar_path_helper() {
        let path = sidecar_path(Path::new("~/.sikil/repo/my-skill"));
        assert_eq!(
            path,
            PathBuf::from("~/.sikil/repo/my-skill/.sikil-source.toml")
        );
    }

    // ---- Content hash integration ----

    #[test]
    fn test_content_hash_excludes_sidecar() {
        use crate::core::manifest::compute_content_hash;

        let tmp = TempDir::new().unwrap();
        let skill_dir = create_test_skill_dir(tmp.path(), "skill");

        // Hash without sidecar
        let hash_before = compute_content_hash(&skill_dir).unwrap();

        // Write sidecar
        write_sidecar(
            &skill_dir,
            r#"
version = 1
source = "owner/repo"
resolved_url = "https://github.com/owner/repo.git"
commit = "abc123"
content_hash = "sha256:hash"
installed_at = "2024-01-01T00:00:00Z"
installer = "sikil/0.1.0"
"#,
        );

        // Hash with sidecar — should be identical
        let hash_after = compute_content_hash(&skill_dir).unwrap();
        assert_eq!(hash_before, hash_after);
    }

    #[test]
    fn test_content_hash_identical_to_manifest_hash() {
        use crate::core::manifest::compute_content_hash;

        let tmp = TempDir::new().unwrap();
        let skill_dir = tmp.path().join("skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# My Skill").unwrap();
        fs::write(skill_dir.join("data.txt"), "some content").unwrap();

        // This is the same function used by both provenance and manifest
        let hash = compute_content_hash(&skill_dir).unwrap();

        // Should be deterministic
        let hash2 = compute_content_hash(&skill_dir).unwrap();
        assert_eq!(hash, hash2);
        assert!(hash.starts_with("sha256:"));
    }

    // ---- Timestamp and version helpers ----

    #[test]
    fn test_now_rfc3339_format() {
        let ts = now_rfc3339();
        // Should be parseable as RFC 3339
        let parsed = chrono::DateTime::parse_from_rfc3339(&ts);
        assert!(parsed.is_ok(), "Failed to parse '{}' as RFC 3339", ts);
    }

    #[test]
    fn test_installer_version_format() {
        let version = installer_version();
        assert!(
            version.starts_with("sikil/"),
            "Expected 'sikil/' prefix, got: {}",
            version
        );
        // Should contain a semver-like version
        let version_part = version.strip_prefix("sikil/").unwrap();
        assert!(
            version_part.contains('.'),
            "Expected semver format, got: {}",
            version_part
        );
    }

    // ---- Local-path source: empty commit ----

    #[test]
    fn test_sidecar_local_path_source_empty_commit() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = create_test_skill_dir(tmp.path(), "local-skill");

        let sidecar = ProvenanceSidecar {
            version: 1,
            source: "/local/path/to/skill".to_string(),
            resolved_url: "/local/path/to/skill".to_string(),
            commit: String::new(),
            content_hash: "sha256:abc123".to_string(),
            installed_at: now_rfc3339(),
            installer: installer_version(),
        };

        save_sidecar(&skill_dir, &sidecar).unwrap();
        let loaded = load_sidecar(&skill_dir).unwrap().unwrap();
        assert_eq!(loaded.commit, "");
        assert_eq!(loaded.source, "/local/path/to/skill");
    }

    // ---- Roundtrip preserves source verbatim ----

    #[test]
    fn test_sidecar_source_preserved_verbatim() {
        let tmp = TempDir::new().unwrap();
        let skill_dir = create_test_skill_dir(tmp.path(), "skill");

        let sidecar = ProvenanceSidecar {
            version: 1,
            source: "owner/repo".to_string(),
            resolved_url: "https://github.com/owner/repo.git".to_string(),
            commit: "abc123".to_string(),
            content_hash: "sha256:hash".to_string(),
            installed_at: "2024-01-01T00:00:00Z".to_string(),
            installer: "sikil/0.1.0".to_string(),
        };

        save_sidecar(&skill_dir, &sidecar).unwrap();
        let loaded = load_sidecar(&skill_dir).unwrap().unwrap();
        assert_eq!(loaded.source, "owner/repo");
    }
}
