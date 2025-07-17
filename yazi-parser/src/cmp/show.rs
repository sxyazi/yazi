use std::{ffi::OsString, path::{MAIN_SEPARATOR_STR, PathBuf}};

use yazi_shared::{Id, SStr, event::CmdCow};

#[derive(Default)]
pub struct ShowOpt {
	pub cache:      Vec<CmpItem>,
	pub cache_name: PathBuf,
	pub word:       SStr,
	pub ticket:     Id,
}

impl TryFrom<CmdCow> for ShowOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		if let Some(opt) = c.take_any2("opt") {
			return opt;
		}

		Ok(Self {
			cache:      c.take_any("cache").unwrap_or_default(),
			cache_name: c.take_any("cache-name").unwrap_or_default(),
			word:       c.take_str("word").unwrap_or_default(),
			ticket:     c.id("ticket").unwrap_or_default(),
		})
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
