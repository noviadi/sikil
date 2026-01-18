//! Integration tests for the SKILL.md parser
//!
//! These tests use actual fixture files to verify the parser works correctly
//! with real-world SKILL.md files.

use sikil::core::parser::parse_skill_md;
use std::path::PathBuf;

/// Returns the path to the fixtures directory
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("skills")
}

#[test]
fn test_parse_valid_skill_minimal() {
    let fixture_path = fixtures_dir().join("valid").join("minimal-skill.md");

    let result = parse_skill_md(&fixture_path);

    assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result.err());

    let metadata = result.unwrap();
    assert_eq!(metadata.name, "minimal-skill");
    assert_eq!(
        metadata.description,
        "A minimal skill with only required fields"
    );
    assert!(metadata.version.is_none());
    assert!(metadata.author.is_none());
    assert!(metadata.license.is_none());
}

#[test]
fn test_parse_valid_skill_complete() {
    let fixture_path = fixtures_dir().join("valid").join("complete-skill.md");

    let result = parse_skill_md(&fixture_path);

    assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result.err());

    let metadata = result.unwrap();
    assert_eq!(metadata.name, "complete-skill");
    assert_eq!(
        metadata.description,
        "A skill with all optional fields populated"
    );
    assert_eq!(metadata.version, Some("1.0.0".to_string()));
    assert_eq!(metadata.author, Some("Jane Developer".to_string()));
    assert_eq!(metadata.license, Some("MIT".to_string()));
}

#[test]
fn test_parse_valid_skill_complex_metadata() {
    let fixture_path = fixtures_dir().join("valid").join("complex-metadata.md");

    let result = parse_skill_md(&fixture_path);

    assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result.err());

    let metadata = result.unwrap();
    assert_eq!(metadata.name, "complex-metadata");
    // Multi-line description
    assert!(metadata.description.contains("multi-line"));
    assert!(metadata.description.contains("spans multiple"));
    assert_eq!(metadata.version, Some("2.3.4-beta".to_string()));
    assert_eq!(
        metadata.author,
        Some("Open Source Contributor <contributor@example.com>".to_string())
    );
    assert_eq!(metadata.license, Some("Apache-2.0".to_string()));
}

#[test]
fn test_parse_valid_skill_simple() {
    let fixture_path = fixtures_dir().join("valid").join("simple-skill.md");

    let result = parse_skill_md(&fixture_path);

    assert!(result.is_ok(), "Expected Ok, got Err: {:?}", result.err());

    let metadata = result.unwrap();
    assert_eq!(metadata.name, "simple-skill");
    assert_eq!(metadata.description, "A simple skill for testing");
    assert_eq!(metadata.version, Some("0.1.0".to_string()));
    assert!(metadata.author.is_none());
    assert!(metadata.license.is_none());
}

#[test]
fn test_parse_invalid_missing_name() {
    let fixture_path = fixtures_dir().join("invalid").join("missing-name.md");

    let result = parse_skill_md(&fixture_path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("missing required field 'name'"),
        "Expected 'missing required field' error, got: {}",
        err_msg
    );
}

#[test]
fn test_parse_invalid_missing_description() {
    let fixture_path = fixtures_dir()
        .join("invalid")
        .join("missing-description.md");

    let result = parse_skill_md(&fixture_path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("missing required field 'description'"),
        "Expected 'missing required field' error, got: {}",
        err_msg
    );
}

#[test]
fn test_parse_invalid_no_frontmatter() {
    let fixture_path = fixtures_dir().join("invalid").join("no-frontmatter.md");

    let result = parse_skill_md(&fixture_path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("missing frontmatter delimiters") || err_msg.contains("frontmatter"),
        "Expected frontmatter error, got: {}",
        err_msg
    );
}

#[test]
fn test_parse_invalid_single_delimiter() {
    let fixture_path = fixtures_dir().join("invalid").join("single-delimiter.md");

    let result = parse_skill_md(&fixture_path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("only one '---' marker") || err_msg.contains("malformed"),
        "Expected malformed frontmatter error, got: {}",
        err_msg
    );
}

#[test]
fn test_parse_invalid_invalid_yaml() {
    let fixture_path = fixtures_dir().join("invalid").join("invalid-yaml.md");

    let result = parse_skill_md(&fixture_path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("failed to parse YAML") || err_msg.contains("YAML"),
        "Expected YAML parse error, got: {}",
        err_msg
    );
}

#[test]
fn test_parse_invalid_frontmatter_not_at_start() {
    let fixture_path = fixtures_dir()
        .join("invalid")
        .join("frontmatter-not-at-start.md");

    let result = parse_skill_md(&fixture_path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("frontmatter must be at the start") || err_msg.contains("start"),
        "Expected 'frontmatter must be at the start' error, got: {}",
        err_msg
    );
}

// Snapshot tests for JSON output

#[test]
fn test_snapshot_metadata_minimal() {
    let fixture_path = fixtures_dir().join("valid").join("minimal-skill.md");

    let metadata = parse_skill_md(&fixture_path).unwrap();

    // Use insta for snapshot testing of the JSON serialization
    let json = serde_json::to_string_pretty(&metadata).unwrap();
    insta::assert_snapshot!(json);
}

#[test]
fn test_snapshot_metadata_complete() {
    let fixture_path = fixtures_dir().join("valid").join("complete-skill.md");

    let metadata = parse_skill_md(&fixture_path).unwrap();

    let json = serde_json::to_string_pretty(&metadata).unwrap();
    insta::assert_snapshot!(json);
}

#[test]
fn test_snapshot_metadata_complex() {
    let fixture_path = fixtures_dir().join("valid").join("complex-metadata.md");

    let metadata = parse_skill_md(&fixture_path).unwrap();

    let json = serde_json::to_string_pretty(&metadata).unwrap();
    insta::assert_snapshot!(json);
}
