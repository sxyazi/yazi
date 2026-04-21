use anyhow::Result;
use arc_swap::ArcSwap;
use serde::{Deserialize, Deserializer, de};
use yazi_codegen::{DeserializeOver, DeserializeOver2};
use yazi_fs::{SortBy, SortFallback};
use yazi_shim::{arc_swap::IntoPointee, cell::SyncCell};

use super::{MgrRatio, MouseEvents};

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
	#[serde(deserialize_with = "deserialize_linemode")]
	pub linemode:     ArcSwap<String>,
	pub show_hidden:  SyncCell<bool>,
	pub show_symlink: SyncCell<bool>,
	pub scrolloff:    SyncCell<u8>,
	pub mouse_events: SyncCell<MouseEvents>,
}

fn deserialize_linemode<'de, D>(deserializer: D) -> Result<ArcSwap<String>, D::Error>
where
	D: Deserializer<'de>,
{
	let s = String::deserialize(deserializer)?;
	if s.is_empty() || s.len() > 20 {
		return Err(de::Error::custom("linemode must be between 1 and 20 characters."));
	}

	Ok(s.into_pointee())
}
