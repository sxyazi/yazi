use anyhow::{Result, bail};
use serde::Deserialize;
use yazi_codegen::DeserializeOver2;
use yazi_fs::{CWD, SortBy};

use super::{MgrRatio, MouseEvents};

#[derive(Debug, Deserialize, DeserializeOver2)]
pub struct Mgr {
	pub ratio: MgrRatio,

	// Sorting
	pub sort_by:        SortBy,
	pub sort_sensitive: bool,
	pub sort_reverse:   bool,
	pub sort_dir_first: bool,
	pub sort_translit:  bool,

	// Display
	pub linemode:     String,
	pub show_hidden:  bool,
	pub show_symlink: bool,
	pub scrolloff:    u8,
	pub mouse_events: MouseEvents,
	pub title_format: String,
}

impl Mgr {
	pub fn title(&self) -> Option<String> {
		if self.title_format.is_empty() {
			return None;
		}

		let home = dirs::home_dir().unwrap_or_default();
		let cwd = if let Ok(p) = CWD.load().strip_prefix(home) {
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
