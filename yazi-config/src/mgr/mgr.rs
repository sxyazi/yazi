use anyhow::{Result, bail};
use serde::Deserialize;
use yazi_codegen::DeserializeOver2;
use yazi_fs::{CWD, SortBy};
use yazi_shared::{SyncCell, url::UrlLike};

use super::{MgrRatio, MouseEvents};

#[derive(Debug, Deserialize, DeserializeOver2)]
pub struct Mgr {
	pub ratio: SyncCell<MgrRatio>,

	// Sorting
	pub sort_by:        SyncCell<SortBy>,
	pub sort_sensitive: SyncCell<bool>,
	pub sort_reverse:   SyncCell<bool>,
	pub sort_dir_first: SyncCell<bool>,
	pub sort_translit:  SyncCell<bool>,

	// Display
	pub linemode:     String,
	pub show_hidden:  SyncCell<bool>,
	pub show_symlink: SyncCell<bool>,
	pub scrolloff:    SyncCell<u8>,
	pub mouse_events: SyncCell<MouseEvents>,
	pub title_format: String,
}

impl Mgr {
	pub fn title(&self) -> Option<String> {
		if self.title_format.is_empty() {
			return None;
		}

		let home = dirs::home_dir().unwrap_or_default();
		let cwd = if let Ok(p) = CWD.load().try_strip_prefix(home) {
			format!("~{}{}", std::path::MAIN_SEPARATOR, p.display())
		} else {
			format!("{}", CWD.load().display())
		};

		Some(self.title_format.replace("{cwd}", &cwd))
	}

	pub(crate) fn reshape(self) -> Result<Self> {
		if self.linemode.is_empty() || self.linemode.len() > 20 {
			bail!("[mgr].linemode must be between 1 and 20 characters.");
		}

		Ok(self)
	}
}
