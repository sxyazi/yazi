use anyhow::{Result, bail};
use serde::Deserialize;
use yazi_codegen::DeserializeOver2;
use yazi_fs::SortBy;
use yazi_shared::SyncCell;

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
}

impl Mgr {
	pub(crate) fn reshape(self) -> Result<Self> {
		if self.linemode.is_empty() || self.linemode.len() > 20 {
			bail!("[mgr].linemode must be between 1 and 20 characters.");
		}

		Ok(self)
	}
}
