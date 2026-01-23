//! Core module containing fundamental data types and models

pub mod cache;
pub mod config;
pub mod conflicts;
pub mod errors;
pub mod parser;
pub mod scanner;
pub mod skill;

pub use cache::{Cache, JsonCache, ScanEntry};
pub use config::{AgentConfig, Config};
pub use conflicts::{
    detect_conflicts, filter_error_conflicts, Conflict, ConflictLocation, ConflictType,
};
pub use errors::SikilError;
pub use parser::{extract_frontmatter, parse_skill_md, validate_skill_name};
pub use scanner::{ScanResult, Scanner, SkillEntry};
pub use skill::{Agent, Installation, Scope, Skill, SkillMetadata};
