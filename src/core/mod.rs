//! Core module containing fundamental data types and models

pub mod config;
pub mod errors;
pub mod skill;

pub use config::{AgentConfig, Config};
pub use errors::SikilError;
pub use skill::{Agent, Installation, Scope, Skill, SkillMetadata};
