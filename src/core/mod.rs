//! Core module containing fundamental data types and models

pub mod config;
pub mod errors;
pub mod parser;
pub mod skill;

pub use config::{AgentConfig, Config};
pub use errors::SikilError;
pub use parser::{extract_frontmatter, parse_skill_md};
pub use skill::{Agent, Installation, Scope, Skill, SkillMetadata};
