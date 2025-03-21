use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use yazi_codegen::DeserializeOver2;
use yazi_fs::SortBy;

use super::{MgrRatio, MouseEvents};

#[derive(Debug, Deserialize, DeserializeOver2, Serialize)]
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
	pub(crate) fn reshape(self) -> Result<Self> {
		if self.linemode.is_empty() || self.linemode.len() > 20 {
			bail!("[mgr].linemode must be between 1 and 20 characters.");
		}

		Ok(self)
	}
}
