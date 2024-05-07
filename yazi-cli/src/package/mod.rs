#![allow(clippy::module_inception)]

mod add;
mod deploy;
mod git;
mod install;
mod package;
mod parser;
mod upgrade;

use git::*;
pub(super) use package::*;

pub(super) fn init() {
	let root = yazi_shared::Xdg::state_dir().join("packages");
	std::fs::create_dir_all(root).expect("Failed to create packages directory");
}
