//! Core module containing fundamental data types and models

pub mod error;
pub mod skill;

pub use error::SikilError;
pub use skill::{Agent, Installation, Scope, Skill, SkillMetadata};
