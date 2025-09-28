#![allow(clippy::module_inception)]

yazi_macro::mod_flat!(add delete dependency deploy git hash install package upgrade);

use anyhow::Context;
use yazi_fs::Xdg;

pub(super) fn init() -> anyhow::Result<()> {
	let root = Xdg::state_dir().join("packages");
	std::fs::create_dir_all(&root)
		.with_context(|| format!("failed to create packages directory: {root:?}"))?;

	Ok(())
}
