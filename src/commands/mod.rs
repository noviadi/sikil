//! Commands module for implementing CLI subcommands

pub mod adopt;
pub mod agent_selection;
pub mod install;
pub mod list;
pub mod remove;
pub mod show;
pub mod unmanage;
pub mod validate;

pub use adopt::{execute_adopt, AdoptArgs};
pub use agent_selection::{parse_agent_selection, prompt_agent_selection};
pub use install::{execute_install_git, execute_install_local, InstallArgs};
pub use list::{execute_list, ListArgs};
pub use remove::{execute_remove, RemoveArgs};
pub use show::{execute_show, ShowArgs};
pub use unmanage::{execute_unmanage, UnmanageArgs};
pub use validate::{execute_validate, ValidateArgs};
