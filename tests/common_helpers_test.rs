//! Integration tests for common test helpers
//!
//! This file validates that the common test utilities work correctly.

mod common;

use common::*;

#[test]
fn integration_test_temp_skill_setup() {
    let temp_dir = setup_temp_skill_dir();
    assert!(temp_dir.path().exists());
}

#[test]
fn integration_test_skill_dir_creation() {
    let temp_dir = setup_temp_skill_dir();
    let skill_path = create_skill_dir(temp_dir.path(), "my-skill");
    assert!(skill_path.exists());
    assert!(!skill_path.join("SKILL.md").exists()); // No SKILL.md yet
}

#[test]
fn integration_test_skill_md_creation() {
    let temp_dir = setup_temp_skill_dir();
    let skill_path = create_skill_dir(temp_dir.path(), "test-skill");

    create_minimal_skill_md(&skill_path, "test-skill", "Test description");

    let skill_md = skill_path.join("SKILL.md");
    assert!(skill_md.exists());

    let content = std::fs::read_to_string(&skill_md).unwrap();
    assert!(content.contains("name: \"test-skill\""));
    assert!(content.contains("description: \"Test description\""));
    assert!(content.contains("---"));
}

#[test]
fn integration_test_complete_skill_md() {
    let temp_dir = setup_temp_skill_dir();
    let skill_path = create_skill_dir(temp_dir.path(), "full-skill");

    create_complete_skill_md(&skill_path, "full-skill", "Full test skill");

    let skill_md = skill_path.join("SKILL.md");
    let content = std::fs::read_to_string(&skill_md).unwrap();

    assert!(content.contains("name: \"full-skill\""));
    assert!(content.contains("version: \"0.1.0\""));
    assert!(content.contains("author: \"Test Author\""));
    assert!(content.contains("license: \"MIT\""));
}
