#![allow(clippy::module_inception)]

mod add;
mod deploy;
mod git;
mod install;
mod package;
mod parser;
mod upgrade;

use anyhow::Context;
use git::*;
pub(super) use package::*;

pub(super) fn init() -> anyhow::Result<()> {
	let root = yazi_shared::Xdg::state_dir().join("packages");
	std::fs::create_dir_all(&root)
		.with_context(|| format!("failed to create packages directory: {root:?}"))?;

	Ok(())
}
