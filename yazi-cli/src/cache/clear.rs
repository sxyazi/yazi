use yazi_config::YAZI;
use yazi_fs::Xdg;
use yazi_macro::outln;

use crate::cache::Cache;

impl Cache {
	pub(crate) fn clear() -> anyhow::Result<()> {
		let path = &YAZI.preview.cache_dir;

		if path == Xdg::temp_dir() {
			outln!("Clearing cache directory: \n{path:?}")?;
			std::fs::remove_dir_all(path)?;
		} else {
			outln!(
				"You've changed the default cache directory, for your data's safety, please clear it manually: \n{path:?}"
			)?;
		}

		Ok(())
	}
}
