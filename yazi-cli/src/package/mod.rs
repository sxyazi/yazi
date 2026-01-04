yazi_macro::mod_flat!(add delete dependency deploy git hash install package upgrade);

use anyhow::Context;
use yazi_fs::Xdg;

pub(super) fn init() -> anyhow::Result<()> {
	let packages_dir = Xdg::state_dir().join("packages");
	std::fs::create_dir_all(&packages_dir)
		.with_context(|| format!("failed to create packages directory: {packages_dir:?}"))?;

	let config_dir = Xdg::config_dir();
	std::fs::create_dir_all(&config_dir)
		.with_context(|| format!("failed to create config directory: {config_dir:?}"))?;

	Ok(())
}
