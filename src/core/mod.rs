//! Core module containing fundamental data types and models

pub mod cache;
pub mod config;
pub mod errors;
pub mod parser;
pub mod scanner;
pub mod skill;

pub use cache::{Cache, ScanEntry, SqliteCache};
pub use config::{AgentConfig, Config};
pub use errors::SikilError;
pub use parser::{extract_frontmatter, parse_skill_md, validate_skill_name};
pub use scanner::{ScanResult, Scanner, SkillEntry};
pub use skill::{Agent, Installation, Scope, Skill, SkillMetadata};
