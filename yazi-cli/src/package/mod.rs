#![allow(clippy::module_inception)]

yazi_macro::mod_flat!(add, deploy, git, install, package, parser, upgrade);

use anyhow::Context;

pub(super) fn init() -> anyhow::Result<()> {
	let root = yazi_shared::Xdg::state_dir().join("packages");
	std::fs::create_dir_all(&root)
		.with_context(|| format!("failed to create packages directory: {root:?}"))?;

	Ok(())
}
