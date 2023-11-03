#[allow(clippy::module_inception)]
mod child;
mod init;
mod message;
mod parent;
mod process;

pub mod fork;
pub use process::Process;
