use std::{ffi::OsString, path::MAIN_SEPARATOR_STR};

use anyhow::bail;
use yazi_shared::{Id, event::CmdCow, url::{Url, UrnBuf}};

#[derive(Default)]
pub struct ShowOpt {
	pub cache:      Vec<CmpItem>,
	pub cache_name: Url,
	pub word:       UrnBuf,
	pub ticket:     Id,
}

impl TryFrom<CmdCow> for ShowOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		if let Some(opt) = c.take_any2("opt") {
			opt
		} else {
			bail!("missing 'opt' argument");
		}
	}
}

// --- Item
#[derive(Debug, Clone)]
pub struct CmpItem {
	pub name:   OsString,
	pub is_dir: bool,
}

impl CmpItem {
	pub fn completable(&self) -> String {
		format!("{}{}", self.name.to_string_lossy(), if self.is_dir { MAIN_SEPARATOR_STR } else { "" })
	}
}
