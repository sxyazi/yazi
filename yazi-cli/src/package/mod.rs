#![allow(clippy::module_inception)]

mod add;
mod config;
mod deploy;
mod git;
mod install;
mod package;
mod parser;
mod upgrade;

use git::*;
pub(super) use package::*;
pub(super) use parser::*;

pub(super) fn init() {
	let root = yazi_shared::Xdg::state_dir().join("packages");
	std::fs::create_dir_all(root).expect("Failed to create packages directory");
}
