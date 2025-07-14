use std::path::PathBuf;

use yazi_proxy::options::CmpItem;
use yazi_shared::{Id, SStr, event::{Cmd, CmdCow}};

#[derive(Default)]
pub struct ShowOpt {
	pub cache:      Vec<CmpItem>,
	pub cache_name: PathBuf,
	pub word:       SStr,
	pub ticket:     Id,
}

impl From<CmdCow> for ShowOpt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			cache:      c.take_any("cache").unwrap_or_default(),
			cache_name: c.take_any("cache-name").unwrap_or_default(),
			word:       c.take_str("word").unwrap_or_default(),
			ticket:     c.id("ticket").unwrap_or_default(),
		}
	}
}

impl From<Cmd> for ShowOpt {
	fn from(c: Cmd) -> Self { Self::from(CmdCow::from(c)) }
}
