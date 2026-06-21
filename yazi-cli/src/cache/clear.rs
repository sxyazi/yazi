use yazi_config::YAZI;
use yazi_fs::Xdg;

use crate::cache::Cache;

impl Cache {
	pub(crate) fn clear() -> anyhow::Result<()> {
		if YAZI.preview.cache_dir == *Xdg::temp_dir() {
			println!("Clearing cache directory: \n{:?}", YAZI.preview.cache_dir);
			std::fs::remove_dir_all(&YAZI.preview.cache_dir)?;
		} else {
			println!(
				"You've changed the default cache directory, for your data's safety, please clear it manually: \n{:?}",
				YAZI.preview.cache_dir
			);
		}

		Ok(())
	}
}
