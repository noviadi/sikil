//! Core module containing fundamental data types and models

pub mod errors;
pub mod skill;

pub use errors::SikilError;
pub use skill::{Agent, Installation, Scope, Skill, SkillMetadata};
