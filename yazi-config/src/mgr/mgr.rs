use anyhow::Result;
use serde::Deserialize;
use yazi_codegen::{DeserializeOver, DeserializeOver2};
use yazi_fs::{SortBy, SortFallback};
use yazi_shared::SyncCell;

use super::{MgrRatio, MouseEvents};
use crate::mgr::MgrLinemode;

#[derive(Debug, Deserialize, DeserializeOver, DeserializeOver2)]
pub struct Mgr {
	pub ratio: SyncCell<MgrRatio>,

	// Sorting
	pub sort_by:        SyncCell<SortBy>,
	pub sort_sensitive: SyncCell<bool>,
	pub sort_reverse:   SyncCell<bool>,
	pub sort_dir_first: SyncCell<bool>,
	pub sort_translit:  SyncCell<bool>,
	pub sort_fallback:  SyncCell<SortFallback>,

	// Display
	pub linemode:     MgrLinemode,
	pub show_hidden:  SyncCell<bool>,
	pub show_symlink: SyncCell<bool>,
	pub scrolloff:    SyncCell<u8>,
	pub mouse_events: SyncCell<MouseEvents>,
}
