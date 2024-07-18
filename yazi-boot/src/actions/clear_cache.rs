use yazi_config::PREVIEW;
use yazi_shared::Xdg;

use super::Actions;

impl Actions {
	pub(super) fn clear_cache() {
		if PREVIEW.cache_dir == Xdg::cache_dir() {
			println!("Clearing cache directory: \n{:?}", PREVIEW.cache_dir);
			std::fs::remove_dir_all(&PREVIEW.cache_dir).unwrap();
		} else {
			println!(
				"You've changed the default cache directory, for your data's safety, please clear it manually: \n{:?}",
				PREVIEW.cache_dir
			);
		}
	}
}
