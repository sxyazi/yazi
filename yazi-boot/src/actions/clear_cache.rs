use yazi_config::YAZI;
use yazi_fs::Xdg;

use super::Actions;

impl Actions {
	pub(super) fn clear_cache() {
		if YAZI.preview.cache_dir == Xdg::cache_dir() {
			println!("Clearing cache directory: \n{:?}", YAZI.preview.cache_dir);
			std::fs::remove_dir_all(&YAZI.preview.cache_dir).unwrap();
		} else {
			println!(
				"You've changed the default cache directory, for your data's safety, please clear it manually: \n{:?}",
				YAZI.preview.cache_dir
			);
		}
	}
}
