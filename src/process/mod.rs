#[allow(clippy::module_inception)]
mod child;
mod parent;

pub mod fork;
pub use process::Process;
